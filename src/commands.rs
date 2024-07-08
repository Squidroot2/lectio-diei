use log::*;

use crate::args::ConfigCommand;
use crate::config::Config;
use crate::error::{ApplicationError, ArgumentError, DatabaseError, DatabaseGetError, DatabaseInitError, InitConfigError};
use crate::{
    args::{DatabaseCommand, DisplayReadingsArgs},
    date::DateId,
    db::DatabaseHandle,
    orchestration,
};

/// Command: display
///
/// Displays a day, either today or the given one.
pub async fn display(maybe_date_string: Option<String>, readings: DisplayReadingsArgs) -> Result<(), ApplicationError> {
    let date_id = match maybe_date_string {
        Some(date_string) => DateId::checked_from_str(&date_string).map_err(ArgumentError::InvalidDate)?,
        None => {
            let today = DateId::today();
            info!("No date specified. Using '{}'", today);
            today
        }
    };

    orchestration::retrieve_and_display(date_id)
        .await
        .map_err(ApplicationError::RetrievalError)
}

/// Command: db
pub async fn handle_db_command(command: DatabaseCommand) -> Result<(), ApplicationError> {
    match command {
        DatabaseCommand::Remove { dates } => remove_entries(dates).await.map_err(ApplicationError::from),
        DatabaseCommand::Count => count_entries().await.map_err(ApplicationError::from),
    }
}

/// Command: config
pub fn handle_config_command(command: ConfigCommand) -> Result<(), ApplicationError> {
    match command {
        ConfigCommand::Init { force } => init_config(force).map_err(ApplicationError::from),
    }
}

/// Subcommand: db count
///
/// Counts number of lectionaries and prints that to STDOUT
async fn count_entries() -> Result<(), DatabaseError> {
    let db = DatabaseHandle::new().await?;
    let count = db.get_lectionary_count().await.map_err(DatabaseGetError::from)?;

    Ok(println!("{}", count))
}

/// Subcommand: db remove
///
/// Removes a list of entries. Sends removed count to STDOUT
async fn remove_entries(date_strings: Vec<String>) -> Result<(), DatabaseInitError> {
    let date_ids: Vec<DateId> = date_strings
        .iter()
        .filter_map(|date_string| match DateId::checked_from_str(date_string) {
            Ok(date_id) => Some(date_id),
            Err(_) => {
                warn!("'{}' is not a valid date id. Skipping...", date_string);
                None
            }
        })
        .collect();

    let db = DatabaseHandle::new().await?;
    let mut removed_count = 0;
    for id in date_ids {
        let remove_result = db.remove_lectionary(&id).await;
        match remove_result {
            Ok(true) => {
                info!("Successfully removed lectionary '{}'", id);
                removed_count += 1
            }
            Ok(false) => info!("Tried to remove lectionary '{}' but it was not present", id),
            Err(e) => error!("Failed to remove lectionary '{}': {}", id, e),
        };
    }

    Ok(println!("{}", removed_count))
}

/// Subcomand: config init
fn init_config(force: bool) -> Result<(), InitConfigError> {
    match Config::initialize_default_config(force) {
        Ok(()) => Ok(println!("success")),
        Err(e) => {
            if matches!(e, InitConfigError::AlreadyExists(_)) && !force {
                warn!("Config file already exists. Must use '--force' to overwrite existing file");
                // We'll pretend the command was successful here (unless force was specificed in which case something really weird happened)
                return Ok(());
            }
            Err(e)
        }
    }
}
