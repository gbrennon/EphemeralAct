use std::collections::HashMap;

use bytes::Bytes;
use futures_util::StreamExt;

use crate::{
    core::ports::outbound::{ContainerError, ContainerPort, ExecResult, FileEntry, RunnerContext},
    infrastructure::bollard_wrapper::{
        Client, body_full,
        types::{
            CreateExecOptions, DownloadFromContainerOptionsBuilder, InspectContainerOptions,
            LogOutput, RemoveContainerOptions, StartExecOptions, StartExecResults,
            UploadToContainerOptionsBuilder,
        },
    },
};

/// A running Podman container, created by [`PodmanRuntime`].
pub(super) struct PodmanContainer {
    pub(super) client: Client,
    pub(super) container_id: String,
    pub(super) runtime: tokio::runtime::Handle,
    pub(super) runner_context: RunnerContext,
}

impl ContainerPort for PodmanContainer {
    fn exec(
        &self,
        cmd: &[String],
        workdir: Option<&str>,
        env: &HashMap<String, String>,
    ) -> Result<ExecResult, ContainerError> {
        let exec_config = CreateExecOptions {
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            cmd: Some(cmd.to_vec()),
            working_dir: workdir.map(|s| s.to_string()),
            env: if env.is_empty() {
                None
            } else {
                Some(env.iter().map(|(k, v)| format!("{}={}", k, v)).collect())
            },
            ..Default::default()
        };

        self.runtime.block_on(async {
            let exec = self
                .client
                .create_exec(&self.container_id, exec_config)
                .await
                .map_err(|e| {
                    ContainerError::ExecutionFailed(self.container_id.clone(), e.to_string())
                })?;

            let output = self
                .client
                .start_exec(&exec.id, None::<StartExecOptions>)
                .await
                .map_err(|e| {
                    ContainerError::ExecutionFailed(self.container_id.clone(), e.to_string())
                })?;

            match output {
                StartExecResults::Attached { mut output, input } => {
                    let mut stdout = String::new();
                    let mut stderr = String::new();
                    while let Some(chunk) = output.next().await {
                        match chunk {
                            Ok(LogOutput::StdOut { message }) => {
                                stdout.push_str(&String::from_utf8_lossy(&message));
                            }
                            Ok(LogOutput::StdErr { message }) => {
                                stderr.push_str(&String::from_utf8_lossy(&message));
                            }
                            Ok(_) => {}
                            Err(e) => {
                                return Err(ContainerError::ExecutionFailed(
                                    self.container_id.clone(),
                                    e.to_string(),
                                ));
                            }
                        }
                    }
                    // Drop the input stream before the output stream to signal
                    // EOF to Podman. This prevents crun from flushing debug
                    // errors when the upgraded connection closes.
                    drop(input);

                    // Inspect the exec instance to get the real exit code
                    let exit_code = self
                        .client
                        .inspect_exec(&exec.id)
                        .await
                        .map(|inspect| inspect.exit_code.unwrap_or(0))
                        .unwrap_or(0);

                    Ok(ExecResult {
                        exit_code,
                        stdout,
                        stderr,
                    })
                }
                StartExecResults::Detached => Ok(ExecResult {
                    exit_code: 0,
                    stdout: String::new(),
                    stderr: String::new(),
                }),
            }
        })
    }

    fn copy_to(&self, container_path: &str, entries: &[FileEntry]) -> Result<(), ContainerError> {
        let mut tar_builder = tar::Builder::new(Vec::new());

        for entry in entries {
            let mut header = tar::Header::new_gnu();
            header.set_path(&entry.path).map_err(|e| {
                ContainerError::CopyFailed(self.container_id.clone(), e.to_string())
            })?;
            header.set_size(entry.content.len() as u64);
            header.set_mode(entry.mode);
            header.set_cksum();

            tar_builder
                .append(&header, entry.content.as_slice())
                .map_err(|e| {
                    ContainerError::CopyFailed(self.container_id.clone(), e.to_string())
                })?;
        }

        let tar_data = tar_builder
            .into_inner()
            .map_err(|e| ContainerError::CopyFailed(self.container_id.clone(), e.to_string()))?;

        let upload_options = UploadToContainerOptionsBuilder::new()
            .path(container_path)
            .build();

        self.runtime.block_on(async {
            self.client
                .upload_to_container(
                    &self.container_id,
                    Some(upload_options),
                    body_full(Bytes::from(tar_data)),
                )
                .await
                .map_err(|e| ContainerError::CopyFailed(self.container_id.clone(), e.to_string()))
        })?;

        Ok(())
    }

    fn copy_from(&self, container_path: &str) -> Result<Vec<FileEntry>, ContainerError> {
        let download_options = DownloadFromContainerOptionsBuilder::new()
            .path(container_path)
            .build();

        self.runtime.block_on(async {
            let mut stream = self
                .client
                .download_from_container(&self.container_id, Some(download_options));

            let mut tar_bytes = Vec::new();
            while let Some(result) = stream.next().await {
                match result {
                    Ok(chunk) => tar_bytes.extend_from_slice(&chunk),
                    Err(e) => {
                        return Err(ContainerError::CopyFailed(
                            self.container_id.clone(),
                            e.to_string(),
                        ));
                    }
                }
            }

            let mut archive = tar::Archive::new(tar_bytes.as_slice());
            let mut entries = Vec::new();

            for entry_result in archive
                .entries()
                .map_err(|e| ContainerError::CopyFailed(self.container_id.clone(), e.to_string()))?
            {
                let mut entry = entry_result.map_err(|e| {
                    ContainerError::CopyFailed(self.container_id.clone(), e.to_string())
                })?;

                let path = entry
                    .path()
                    .map_err(|e| {
                        ContainerError::CopyFailed(self.container_id.clone(), e.to_string())
                    })?
                    .to_string_lossy()
                    .to_string();

                let mode = entry.header().mode().map_err(|e| {
                    ContainerError::CopyFailed(self.container_id.clone(), e.to_string())
                })?;

                let mut content = Vec::new();
                std::io::Read::read_to_end(&mut entry, &mut content).map_err(|e| {
                    ContainerError::CopyFailed(self.container_id.clone(), e.to_string())
                })?;

                entries.push(FileEntry {
                    path,
                    content,
                    mode,
                });
            }

            Ok(entries)
        })
    }

    fn remove(&self) -> Result<(), ContainerError> {
        self.runtime.block_on(async {
            self.client
                .remove_container(
                    &self.container_id,
                    Some(RemoveContainerOptions {
                        force: true,
                        ..Default::default()
                    }),
                )
                .await
                .map_err(|e| {
                    ContainerError::RemovalFailed(self.container_id.clone(), e.to_string())
                })
        })
    }

    fn get_runner_context(&self) -> Result<RunnerContext, ContainerError> {
        self.runtime.block_on(async {
            let info = self
                .client
                .inspect_container(&self.container_id, None::<InspectContainerOptions>)
                .await
                .map_err(|_| ContainerError::NotFound(self.container_id.clone()))?;

            let env_map: HashMap<String, String> = info
                .config
                .and_then(|c| c.env)
                .unwrap_or_default()
                .iter()
                .filter_map(|kv| {
                    let mut parts = kv.splitn(2, '=');
                    Some((
                        parts.next()?.to_string(),
                        parts.next().unwrap_or("").to_string(),
                    ))
                })
                .collect();

            let mut ctx = self.runner_context.clone();
            ctx.env.extend(env_map);
            Ok(ctx)
        })
    }
}
