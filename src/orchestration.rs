use std::error::Error;

use log::{error, info, warn};

use crate::client::WebClient;
use crate::date::DateId;
use crate::db::Database;
use crate::lectionary::Lectionary;

pub async fn retrieve_and_display(date_id: DateId) -> Result<(), Box<dyn Error>> {
    let lectionary = retrieve_lectionary(date_id).await?;
    lectionary.pretty_print();
    Ok(())
}

async fn retrieve_lectionary(date_id: DateId) -> Result<Lectionary, Box<dyn Error>> {
    let db = Database::new().await?;
    //TODO handle case where db init fails
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
                    return Err(web_error);
                }
            }
        }
    };

    Ok(lectionary)
}
