use std::process::ExitCode;

use clap::Parser;
use log::*;

use lectio_diei::args::{Arguments, Command};
use lectio_diei::commands::{self, ApplicationError};
use lectio_diei::logging;

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(e) = run().await {
        error!("{}", e);
        return ExitCode::from(e.exit_code());
    }

    ExitCode::SUCCESS
}

async fn run() -> Result<(), ApplicationError> {
    logging::init_logger();
    let args = Arguments::parse();

    match args.command {
        Command::Display { date, readings } => commands::display(date, readings).await,
        Command::Db { command } => commands::handle_db_command(command).await,
        Command::Config {} => Err(ApplicationError::NotImplemented),
    }
}
