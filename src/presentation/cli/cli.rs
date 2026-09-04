use std::{ffi::OsString, io::Write};

use clap::Parser;

use super::{
    cli_parser::CliParser, command::Command, list_actions_handler::ListActionsHandler,
    list_workflows_handler::ListWorkflowsHandler, run_handler::RunHandler,
};
use crate::application::ports::inbound::{
    list_actions_port::ListActionsPort, list_workflows_port::ListWorkflowsPort,
    run_all_workflows_port::RunAllWorkflowsPort, run_workflow_port::RunWorkflowPort,
};

pub struct Cli {
    run_workflow_port: Box<dyn RunWorkflowPort>,
    run_all_workflows_port: Box<dyn RunAllWorkflowsPort>,
    list_workflows_port: Box<dyn ListWorkflowsPort>,
    list_actions_port: Box<dyn ListActionsPort>,
}

impl Cli {
    pub fn new(
        run_workflow_port: Box<dyn RunWorkflowPort>,
        run_all_workflows_port: Box<dyn RunAllWorkflowsPort>,
        list_workflows_port: Box<dyn ListWorkflowsPort>,
        list_actions_port: Box<dyn ListActionsPort>,
    ) -> Self {
        Self {
            run_workflow_port,
            run_all_workflows_port,
            list_workflows_port,
            list_actions_port,
        }
    }

    pub fn run<I, T>(self, args: I) -> Result<(), Box<dyn std::error::Error>>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let parsed = CliParser::try_parse_from(args);
        let cli = match parsed {
            Ok(cli) => cli,
            Err(e) => {
                if e.kind() == clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand {
                    let mut stdout = std::io::stdout();
                    let _ = write!(stdout, "{e}");
                    let _ = stdout.flush();
                    return Ok(());
                }
                let _ = write!(std::io::stderr(), "{e}");
                return Err(e.to_string().into());
            }
        };
        match cli.command {
            Command::Run(args) => RunHandler::handle(
                *args,
                &*self.run_workflow_port,
                &*self.run_all_workflows_port,
            ),
            Command::ListWorkflows(args) => {
                ListWorkflowsHandler::handle(*args, &*self.list_workflows_port)
            }
            Command::ListActions(args) => {
                ListActionsHandler::handle(*args, &*self.list_actions_port)
            }
        }
    }
}
