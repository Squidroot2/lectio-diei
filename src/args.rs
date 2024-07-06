use std::{error::Error, fmt::Display};

use chrono::format::ParseError;
use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Print the Reading to STDOUT
    Display {
        /// Date to retrieve (Uses today if not specified)
        #[arg(short, long)]
        date: Option<String>,

        #[command(flatten)]
        readings: DisplayReadingsArgs,
    },
    /// Manage the database, including retrieving more readings//TODO
    Db {
        #[command(subcommand)]
        command: DatabaseCommand,
    },
    /// View and change the config//TODO
    Config {},
}

#[derive(Subcommand)]
pub enum DatabaseCommand {
    Remove {
        /// Dates to remove
        #[arg(trailing_var_arg(true), num_args(1..usize::MAX))]
        dates: Vec<String>,
    },
}

#[derive(Args)]
#[group(required = false, multiple = false)]
pub struct DisplayReadingsArgs {
    /// Displays the readings in the specified order
    #[arg(short, long, value_enum)]
    readings: Option<Vec<ReadingArg>>,

    /// Displays all readings in default order
    #[arg(short, long)]
    all: bool,

    /// Only display the name of the day
    #[arg(long)]
    day_only: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum ReadingArg {
    Reading1,
    Psalm,
    Reading2,
    Gospel,
}

#[derive(Debug)]
pub enum ArgumentError {
    InvalidDate(ParseError),
}

impl Display for ArgumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDate(e) => write!(f, "Invalid date Argument: {}", e),
        }
    }
}

impl Error for ArgumentError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidDate(e) => Some(e),
        }
    }
}
