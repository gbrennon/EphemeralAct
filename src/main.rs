use std::{
    io::{BufRead, BufReader, Write},
    os::unix::io::FromRawFd,
    thread::{self, JoinHandle},
};

use ephemeral_act::{infrastructure::Container, presentation::composition_root::CompositionRoot};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Filter cosmetic crun/Podman errors from stderr. Podman 5.8 + crun
    // emits "executable file `bash` not found" and "read unixpacket"
    // errors asynchronously through the Podman socket when exec sessions
    // are cleaned up after container stop. These go directly to stderr
    // through bollard and cannot be intercepted in Rust business logic.
    // We filter them here at the process level.
    let real_stderr = unsafe { libc::dup(libc::STDERR_FILENO) };
    let mut filter_handle: Option<JoinHandle<()>> = None;
    let mut restore_fd: libc::c_int = -1;

    if real_stderr >= 0 {
        restore_fd = real_stderr;
        let mut fds: [libc::c_int; 2] = [0; 2];
        if unsafe { libc::pipe(fds.as_mut_ptr()) } == 0 {
            let read_end = fds[0];
            let write_end = fds[1];
            let filter_fd = unsafe { libc::dup(real_stderr) };
            // Spawn a thread that copies from the pipe to real stderr,
            // dropping lines that match crun error patterns.
            filter_handle = Some(thread::spawn(move || {
                let reader = BufReader::new(unsafe { std::fs::File::from_raw_fd(read_end) });
                let mut real = unsafe { std::fs::File::from_raw_fd(filter_fd) };
                for line in reader.lines().map_while(Result::ok) {
                    if line.starts_with("Error: crun: executable file `bash` not found")
                        || line.starts_with("Error: read unixpacket")
                    {
                        continue;
                    }
                    let _ = writeln!(real, "{}", line);
                    let _ = real.flush();
                }
            }));
            // Redirect our stderr into the pipe.
            unsafe { libc::dup2(write_end, libc::STDERR_FILENO) };
            unsafe { libc::close(write_end) };
        }
    }

    let use_case = Container::build();
    let app = CompositionRoot::compose(use_case);
    let result = app.cli.run(std::env::args_os());
    // This ensures all buffered output (including any late-arriving
    // crun errors) is flushed through the filter before the process
    // terminates.
    if restore_fd >= 0 {
        unsafe { libc::dup2(restore_fd, libc::STDERR_FILENO) };
        unsafe { libc::close(restore_fd) };
    }
    if let Some(handle) = filter_handle {
        let _ = handle.join();
    }

    match result {
        Ok(()) => std::process::exit(0),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
