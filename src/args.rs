use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Command,

    /// Disabled colors for STDERR and STDOUT. Use if terminal does not support colors or redirecting to file
    #[arg(long)]
    pub no_color: bool,
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
    /// Removes specified date(s) from database if present. Writes number removed to STDOUT
    Remove {
        /// Dates to remove. Should be in MMddYY format
        #[arg(trailing_var_arg(true), num_args(1..usize::MAX))]
        dates: Vec<String>,
    },
    /// Gets a count of the rows in the db. Writes num to STDOUT
    Count,
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
