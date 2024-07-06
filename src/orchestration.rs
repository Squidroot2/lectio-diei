use std::error::Error;
use std::fmt::{self, Display};

use log::*;

use crate::client::{WebClient, WebGetError};
use crate::date::DateId;
use crate::db::{DatabaseError, DatabaseHandle};
use crate::lectionary::Lectionary;

/// Retrieves lectionary from db and web and attempts to store it before printing to STDOUT
pub async fn retrieve_and_display(date_id: DateId) -> Result<(), RetrievalError> {
    let lectionary = retrieve_lectionary(date_id).await?;
    lectionary.pretty_print();
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
                    //TODO better error here
                    return Err(RetrievalError::from(web_error));
                }
            }
        }
    };
    Ok(lectionary)
}

#[derive(Debug)]
pub struct RetrievalError {
    db_error: Option<DatabaseError>,
    web_error: Option<WebGetError>,
}

impl Display for RetrievalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.db_error.as_ref(), self.web_error.as_ref()) {
            (Some(db_error), Some(web_error)) => write!(
                f,
                "Failed to retrieve from db ({}) and failed to retrieve from web ({})",
                db_error, web_error
            ),
            (None, Some(web_error)) => write!(f, "Failed to retrieve from web ({})", web_error),
            (Some(db_error), None) => write!(f, "Failed to retrieve from db ({})", db_error),
            (None, None) => write!(f, "Failed to retrieve (undertermined cause)"),
        }
    }
}

impl Error for RetrievalError {}

impl From<DatabaseError> for RetrievalError {
    fn from(value: DatabaseError) -> Self {
        RetrievalError {
            db_error: Some(value),
            web_error: None,
        }
    }
}

impl From<WebGetError> for RetrievalError {
    fn from(value: WebGetError) -> Self {
        RetrievalError {
            db_error: None,
            web_error: Some(value),
        }
    }
}
