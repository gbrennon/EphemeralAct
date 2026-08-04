/// Handles the `help` subcommand, printing a usage guide that wraps the
/// underlying `act` / `act_runner` invocations plus EphemeralAct-specific
/// options.
pub struct HelpHandler;

impl HelpHandler {
    /// Prints the EphemeralAct usage guide to stdout.
    pub fn handle() -> Result<(), Box<dyn std::error::Error>> {
        println!(
            r#"ephemeral-act — Run GitHub Actions locally in ephemeral repositories

USAGE:
    ephemeral-act run [OPTIONS]

OPTIONS:
        --container-engine <ENGINE>      Container engine: podman (default) or docker
        --container-daemon-socket <URI>  Daemon socket URI [default: unix:///run/podman/podman.sock]
        --workflow <FILE>                Workflow file to execute (e.g. ci.yml)
        --job <NAME>                     Specific job name to run
        --event <NAME>                   Triggering event (e.g. push, pull_request)
        --input <KEY=VALUE>              Workflow input (repeatable)
        --secret <SECRET>                Secret to inject (repeatable)
        --extra-arg <ARG>                Extra argument for act / act_runner (repeatable)
        --rm <BOOL>                      Remove container after run [default: true]
        --bind <BOOL>                    Bind-mount working directory [default: true]
        --preserve                       Keep the ephemeral repository after execution

EXAMPLES:
    ephemeral-act run
    ephemeral-act run --workflow ci.yml --job test
    ephemeral-act run --event push --secret TOKEN=abc123
    ephemeral-act run --container-engine docker --rm=false

The `run` subcommand wraps `act` (GitHub Actions) or `act_runner` (Forgejo).
EphemeralAct auto-detects the CI host by inspecting the repository layout.
"#
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_handler_prints_usage() {
        let result = HelpHandler::handle();
        assert!(result.is_ok());
    }
}
