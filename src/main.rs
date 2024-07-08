use std::process::ExitCode;

use clap::Parser;
use lectio_diei::args::{Arguments, Command};
use lectio_diei::commands::{self};
use lectio_diei::error::ApplicationError;
use lectio_diei::logging::{self, LoggingOptions};
use log::*;

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(e) = run().await {
        error!("{}", e);
        return ExitCode::from(e.exit_code());
    }

    ExitCode::SUCCESS
}

async fn run() -> Result<(), ApplicationError> {
    let args = Arguments::parse();
    logging::init_logger(LoggingOptions { no_color: args.no_color });

    match args.command {
        Command::Display { date, readings } => commands::display(date, readings).await,
        Command::Db { command } => commands::handle_db_command(command).await,
        Command::Config { command } => commands::handle_config_command(command),
    }
}
