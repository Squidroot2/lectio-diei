use log::*;
use tokio::task::JoinSet;

use crate::args::{CommonArguments, ConfigCommand};
use crate::client::WebClient;
use crate::config::Config;
use crate::display::DisplaySettings;
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
pub async fn display(
    maybe_date_string: Option<String>,
    readings: DisplayReadingsArgs,
    args: CommonArguments,
) -> Result<(), ApplicationError> {
    let date_id = match maybe_date_string {
        Some(date_string) => DateId::checked_from_str(&date_string).map_err(ArgumentError::InvalidDate)?,
        None => {
            let today = DateId::today();
            info!("No date specified. Using '{}'", today);
            today
        }
    };

    let config = Config::from_file_or_default();
    let settings = DisplaySettings::from_config_and_args(config, readings, args);

    orchestration::retrieve_and_display(date_id, settings)
        .await
        .map_err(ApplicationError::RetrievalError)
}

/// Command: db
pub async fn handle_db_command(subcommand: DatabaseCommand) -> Result<(), ApplicationError> {
    match subcommand {
        DatabaseCommand::Remove { dates } => remove_entries(dates).await.map_err(ApplicationError::from),
        DatabaseCommand::Count => count_entries().await.map_err(ApplicationError::from),
        DatabaseCommand::Update => update_db().await.map_err(ApplicationError::from),
        DatabaseCommand::Show => show_db().await.map_err(ApplicationError::from),
        DatabaseCommand::Purge => purge_db().await.map_err(ApplicationError::from),
    }
}

/// Command: config
pub fn handle_config_command(subcommand: ConfigCommand) -> Result<(), ApplicationError> {
    match subcommand {
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

/// Subcommand db purge
///
/// Removes all rows from the database and writes the number of rows removed
async fn purge_db() -> Result<(), DatabaseError> {
    let db = DatabaseHandle::new().await?;
    let entries_removed = db.remove_all().await.map_err(DatabaseError::DeleteError)?;

    Ok(println!("{}", entries_removed))
}

/// Subcommand: db update
///
/// Retrieves entries from the web and stores in the database
/// Entries retrieved will depend on the config settings
async fn update_db() -> Result<(), DatabaseInitError> {
    let db = DatabaseHandle::new().await?;
    let web_client = WebClient::default();
    let db_config = Config::from_file_or_default().database;
    let date_ids = DateId::get_list(db_config.past_entries, db_config.future_entries);

    let mut tasks = JoinSet::new();
    for id in date_ids.into_iter() {
        let thread_db = db.clone();
        let thread_client = web_client.clone();
        tasks.spawn(async move { orchestration::ensure_stored(id, &thread_db, &thread_client).await });
    }

    let mut count_added = 0;

    while let Some(thread_result) = tasks.join_next().await {
        match thread_result {
            Err(e) => error!("Failed to store a lectionar (Thread panicked!): {}", e),
            Ok(Err(e)) => error!("Failed to store a lectionary: {}", e),
            Ok(Ok(new)) => {
                if new {
                    count_added += 1
                }
            }
        }
    }
    Ok(println!("{}", count_added))
}

/// Subcommand: db show
///
/// Prints each lectionary row from the lectionary table of the database to STDOUT
async fn show_db() -> Result<(), DatabaseError> {
    let db = DatabaseHandle::new().await?;
    let mut rows = db.get_lectionary_rows().await.map_err(DatabaseGetError::from)?;
    rows.sort_unstable();
    for row in rows {
        println!("{} {}", row.id, row.name);
    }
    Ok(())
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
