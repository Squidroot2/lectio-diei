use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Command,

    #[command(flatten)]
    pub common_args: CommonArguments,
}

#[derive(Args)]
pub struct CommonArguments {
    /// Disables colors
    ///
    /// Output for STDERR and STDOUT will not print with ANSI color codes. Useful if terminal does not support colors or redirecting to file
    #[arg(long, global = true)]
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
    /// Manage the database, including retrieving more readings
    Db {
        #[command(subcommand)]
        command: DatabaseCommand,
    },
    /// View and change the config
    //TODO show config, set config settings
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
}

#[derive(Subcommand)]
pub enum DatabaseCommand {
    /// Removes specified date(s) from database if present.
    ///
    /// Writes number removed to STDOUT
    Remove {
        /// Dates to remove. Should be in MMddYY format
        #[arg(trailing_var_arg(true), num_args(1..usize::MAX))]
        dates: Vec<String>,
    },
    /// Gets a count of the rows in the db.
    ///
    /// Writes num to STDOUT
    Count,
    /// Adds entries from the web to the database
    //TODO add arguments to override config
    Update,
    /// Shows all of the lectionary rows in the database
    ///
    /// Prints every row of the lectionary table, sorted by date, as "[date] [name]"
    Show,
    /// Deletes all data in the database
    ///
    /// Writes number of rows removed to STDOUT
    Purge,
    /// Deletes old entries from the database
    ///
    /// Uses values defined in the config
    Clean {
        #[arg[short, long]]
        all: bool,
    },
    //TODO refresh, store
}

#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Initializes the data at the default location
    Init {
        /// Overrides file if it exists
        #[arg(short, long)]
        force: bool,
    },
}

#[derive(Args)]
#[group(required = false, multiple = false)]
pub struct DisplayReadingsArgs {
    /// Displays the readings in the specified order
    #[arg(short, long, value_enum, num_args=1..)]
    pub readings: Option<Vec<ReadingArg>>,

    /// Displays all readings in default order
    #[arg(short, long)]
    pub all: bool,

    /// Only display the name of the day
    #[arg(long)]
    pub day_only: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReadingArg {
    Reading1,
    Psalm,
    Reading2,
    Gospel,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn arguments_works() {
        Arguments::command().debug_assert();
    }
}