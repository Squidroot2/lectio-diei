use log::*;

use crate::client::WebClient;
use crate::date::DateId;
use crate::db::DatabaseHandle;
use crate::display::DisplaySettings;
use crate::error::{DatabaseError, DbUpdateError, RetrievalError};
use crate::lectionary::Lectionary;

/// Retrieves lectionary from db and web and attempts to store it before printing to STDOUT
pub async fn retrieve_and_display(date_id: DateId, settings: DisplaySettings) -> Result<(), RetrievalError> {
    let lectionary = retrieve_lectionary(date_id).await?;
    lectionary.pretty_print(&settings);
    Ok(())
}

/// Attempts to retrieve Lectionary, first from DB and then from web
async fn retrieve_lectionary(date_id: DateId) -> Result<Lectionary, RetrievalError> {
    match DatabaseHandle::new().await {
        Ok(db) => retrieve_and_store(date_id, &db).await,
        //TODO handle case where db init fails
        Err(e) => Err(RetrievalError::from(DatabaseError::from(e))),
    }
}

/// Returns a Lectionary for displaying. First tries the database. If that fails, retrieves from the web and stores in to database.
async fn retrieve_and_store(date_id: DateId, db: &DatabaseHandle) -> Result<Lectionary, RetrievalError> {
    let lectionary = match db.get_lectionary(&date_id).await {
        Ok(lectionary) => {
            info!("lectionary '{}' present in database", date_id);
            lectionary
        }
        Err(db_error) => {
            warn!(
                "Could not find lectionary '{}' in Database ({}); Retrieving from Web",
                &date_id, db_error
            );
            let client = WebClient::default();
            match client.get_for_date_id(date_id).await {
                Ok(lectionary) => {
                    info!("Retrieved lectionary '{}'; Adding to database", lectionary.get_id());
                    if let Err(e) = db.insert_lectionary(&lectionary).await {
                        warn!("Failed to store lectionary '{}' in database: {}", lectionary.get_id(), e);
                    }
                    lectionary
                }
                Err(web_error) => {
                    error!(
                        "Failed to retrieve from web ({}) after failing to retrieve from database",
                        web_error
                    );
                    return Err(RetrievalError::from(web_error));
                }
            }
        }
    };
    Ok(lectionary)
}

/// Stores a lectionary to the database, if it is not stored already
///
/// Returns true if new lectionary was stored, false if no action taken
pub async fn ensure_stored(date_id: DateId, db: &DatabaseHandle, client: &WebClient) -> Result<bool, DbUpdateError> {
    let is_present = match db.lectionary_present(&date_id).await {
        Ok(is_present) => is_present,
        Err(e) => {
            warn!(
                "Could not access database to determine if {} is present ({}). Will attempt web retrieval",
                &date_id, e
            );
            false
        }
    };
    if is_present {
        info!("Lectionary '{}' is already present in the database", date_id);
        Ok(false)
    } else {
        debug!("Retrieving lectionary with id '{}' from web", &date_id);
        retrieve_for_database(date_id, db, client).await.map(|()| true)
    }
}

async fn retrieve_for_database(date_id: DateId, db: &DatabaseHandle, client: &WebClient) -> Result<(), DbUpdateError> {
    match client.get_for_date_id(date_id).await {
        Ok(lectionary) => {
            info!("Retrieved lectionary '{}'; Adding to database", lectionary.get_id());
            match db.insert_lectionary(&lectionary).await.map_err(DbUpdateError::from) {
                Ok(()) => {
                    info!("Successfully stored new lectionary '{}' to database", lectionary.get_id());
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to store lectionary '{}' to database", lectionary.get_id());
                    Err(e)
                }
            }
        }
        Err(web_error) => Err(web_error.into()),
    }
}
