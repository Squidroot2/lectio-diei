use log::*;

use crate::error::{ApplicationError, ArgumentError, DatabaseInitError};
use crate::{
    args::{DatabaseCommand, DisplayReadingsArgs},
    date::DateId,
    db::DatabaseHandle,
    orchestration,
};

/// Command to display a day
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

pub async fn handle_db_command(command: DatabaseCommand) -> Result<(), ApplicationError> {
    match command {
        DatabaseCommand::Remove { dates } => remove_entries(dates).await.map_err(ApplicationError::from),
    }
}

/// Subcommand for db. Removes a list of entries. Sends removed count to STDOUT
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

    println!("{}", removed_count);

    Ok(())
}
