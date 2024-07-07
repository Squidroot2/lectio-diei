use std::process::ExitCode;

use clap::Parser;
use lectio_diei::args::{Arguments, Command};
use lectio_diei::commands::{self};
use lectio_diei::config::Config;
use lectio_diei::error::ApplicationError;
use lectio_diei::logging;
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
    logging::init_logger();
    let args = Arguments::parse();
    let config = Config::from_file_or_default();

    match args.command {
        Command::Display { date, readings } => commands::display(date, readings).await,
        Command::Db { command } => commands::handle_db_command(command).await,
        Command::Config {} => Err(ApplicationError::NotImplemented),
    }
}
