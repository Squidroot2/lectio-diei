use chrono::{Local, TimeDelta};
use log::*;
use tokio::task::JoinSet;

use crate::args::{CommonArguments, ConfigCommand, FormattingArgs};
use crate::client::WebClient;
use crate::config::{Config, DbConfig};
use crate::display::DisplaySettings;
use crate::error::{ApplicationError, ArgumentError, DatabaseError, DatabaseGetError, DatabaseInitError, InitConfigError, ReadConfigError};
use crate::{
    args::{DatabaseCommand, DisplayReadingsArgs},
    date::DateId,
    db::DatabaseHandle,
    orchestration,
};

/// Command: display
///
/// Displays a day, either today or the given one.
/// # Errors
///  Returns an `ApplicationError` if the command encounterd a fatal error
pub async fn display(
    maybe_date_string: Option<String>,
    readings: DisplayReadingsArgs,
    formatting: FormattingArgs,
    args: CommonArguments,
) -> Result<(), ApplicationError> {
    let date_id = if let Some(date_string) = maybe_date_string {
        DateId::checked_from_str(&date_string).map_err(ArgumentError::InvalidDate)?
    } else {
        let today = DateId::today();
        info!("No date specified. Using '{}'", today);
        today
    };

    let config = Config::from_file_or_default();
    let settings = DisplaySettings::from_config_and_args(config, readings, formatting, args);

    orchestration::retrieve_and_display(date_id, settings)
        .await
        .map_err(ApplicationError::RetrievalError)
}

/// Command: db
///
/// # Errors
/// Returns an `ApplicationError` if the command encounterd a fatal error
pub async fn handle_db_command(subcommand: DatabaseCommand) -> Result<(), ApplicationError> {
    match subcommand {
        DatabaseCommand::Remove { dates } => remove_entries(dates).await.map_err(ApplicationError::from),
        DatabaseCommand::Count => count_entries().await.map_err(ApplicationError::from),
        DatabaseCommand::Update => update_db().await.map_err(ApplicationError::from),
        DatabaseCommand::Show => show_db().await.map_err(ApplicationError::from),
        DatabaseCommand::Purge => purge_db().await.map_err(ApplicationError::from),
        DatabaseCommand::Clean { all } => clean_db(all).await.map_err(ApplicationError::from),
        DatabaseCommand::Refresh => refresh_db().await.map_err(ApplicationError::from),
    }
}

/// Command: config
///
/// # Errors
/// Returns an `ApplicationError` if the config encountered a fatal error
pub fn handle_config_command(subcommand: ConfigCommand) -> Result<(), ApplicationError> {
    match subcommand {
        ConfigCommand::Init { force } => init_config(force).map_err(ApplicationError::from),
        ConfigCommand::Upgrade => upgrade_config().map_err(ApplicationError::ReadConfigError),
        ConfigCommand::Show => {
            show_config();
            Ok(())
        },
    }
}

/// Subcommand: db count
///
/// Counts number of lectionaries and prints that to STDOUT
async fn count_entries() -> Result<(), DatabaseError> {
    let db = DatabaseHandle::new().await?;
    let count = db.get_lectionary_count().await.map_err(DatabaseGetError::from)?;

    println!("{count}");
    Ok(())
}

/// Subcommand: db remove
///
/// Removes a list of entries. Sends removed count to STDOUT
async fn remove_entries(date_strings: Vec<String>) -> Result<(), DatabaseInitError> {
    let date_ids: Vec<DateId> = date_strings
        .iter()
        .filter_map(|date_string| {
            if let Ok(date_id) = DateId::checked_from_str(date_string) {
                Some(date_id)
            } else {
                warn!("'{date_string}' is not a valid date id. Skipping...");
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
                info!("Successfully removed lectionary '{id}'");
                removed_count += 1;
            }
            Ok(false) => info!("Tried to remove lectionary '{id}' but it was not present"),
            Err(e) => error!("Failed to remove lectionary '{id}': {e}"),
        };
    }

    println!("{removed_count}");
    Ok(())
}

/// Subcommand: db purge
///
/// Removes all rows from the database and writes the number of rows removed
async fn purge_db() -> Result<(), DatabaseError> {
    let db = DatabaseHandle::new().await?;
    let entries_removed = db.remove_all().await.map_err(DatabaseError::DeleteError)?;

    println!("{entries_removed}");
    Ok(())
}

/// Subcommand: db clean
///
/// Removes rows that are too old in accordance with the config file
/// If all is true, also removes entries that are too far in the future
async fn clean_db(all: bool) -> Result<(), DatabaseError> {
    let db = DatabaseHandle::new().await?;
    let config = Config::from_file_or_default();
    let num_removed = clean_db_inner(&db, config.database, all).await?;

    println!("{num_removed}");
    Ok(())
}

/// Subcommand: db update
///
/// Retrieves entries from the web and stores in the database
/// Entries retrieved will depend on the config settings
async fn update_db() -> Result<(), DatabaseInitError> {
    let db = DatabaseHandle::new().await?;
    let web_client = WebClient::default();
    let db_config = Config::from_file_or_default().database;
    let num_added = update_db_inner(&db, db_config, &web_client).await;

    println!("{num_added}");
    Ok(())
}

/// Subcommand: db refresh
///
/// Performs a clean, and then an update
async fn refresh_db() -> Result<(), DatabaseInitError> {
    let db = DatabaseHandle::new().await?;
    let db_config = Config::from_file_or_default().database;
    let num_removed = match clean_db_inner(&db, db_config.clone(), false).await {
        Ok(num_removed) => num_removed,
        Err(e) => {
            error!("Encounterd error removing entries during refresh: {e}");
            0
        }
    };
    let web_client = WebClient::default();
    let num_added = update_db_inner(&db, db_config, &web_client).await;

    println!("{num_removed}");
    println!("{num_added}");
    Ok(())
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
        Ok(()) => {
            println!("success");
            Ok(())
        },
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

/// Subcommand: config upgrade
fn upgrade_config() -> Result<(), ReadConfigError> {
    let result = Config::upgrade_config();
    if result.is_ok() {
        println!("success");
    }
    result
}

/// Subcommand: config show
fn show_config() {
    let config = Config::from_file_or_default();
    print!("{config}");
}

/// Used by db clean and db refresh
async fn clean_db_inner(db: &DatabaseHandle, db_config: DbConfig, all: bool) -> Result<u64, DatabaseError> {
    let DbConfig {
        past_entries,
        future_entries,
    } = db_config;
    let earliest_date = Local::now() - TimeDelta::days(i64::from(past_entries));

    let latest_date_id: Option<DateId> = if all {
        let latest_date = Local::now() + TimeDelta::days(i64::from(future_entries));
        Some(DateId::from_local_datetime(&latest_date))
    } else {
        None
    };
    let removed_count = db
        .remove_outside_range(DateId::from_local_datetime(&earliest_date), latest_date_id)
        .await
        .map_err(DatabaseError::DeleteError)?;
    Ok(removed_count)
}

/// Used by db udpate and db refresh
async fn update_db_inner(db: &DatabaseHandle, db_config: DbConfig, web_client: &WebClient) -> u64 {
    let date_ids = DateId::get_list(db_config.past_entries, db_config.future_entries);

    let mut tasks = JoinSet::new();
    for id in date_ids {
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
                    count_added += 1;
                }
            }
        }
    }
    count_added
}
