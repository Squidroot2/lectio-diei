use log::*;
use reqwest::{Client, Url};
use scraper::Html;

use crate::date::DateId;
use crate::error::WebGetError;
use crate::lectionary::Lectionary;

/// Client for interacting with the USCCB site
#[derive(Default, Clone)]
pub struct WebClient {
    client: Client,
}

impl WebClient {
    pub async fn get_for_date_id(&self, date_id: DateId) -> Result<Lectionary, WebGetError> {
        let url = Self::url_for_date(&date_id);
        debug!("Sending GET request to {}", url);
        let response = self.client.get(url).send().await.map_err(WebGetError::ClientError)?;
        if !response.status().is_success() {
            return Err(WebGetError::ErrorStatus(response.status()));
        }

        let response_text = response.text().await.map_err(WebGetError::ResponseError)?;
        let document = Html::parse_document(&response_text);
        Lectionary::create_from_html(date_id, &document).map_err(WebGetError::ParseError)
    }

    fn url_for_date(date_id: &DateId) -> Url {
        let url_string = format!("https://bible.usccb.org/bible/readings/{date_id}.cfm");
        Url::parse(&url_string).expect("Formmatted string is valid URL")
    }
}
