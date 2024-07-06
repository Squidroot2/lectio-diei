use std::error::Error;

use log::debug;
use reqwest::{Client, Url};
use scraper::Html;

use crate::date::DateId;
use crate::lectionary::Lectionary;

/// Client for interacting with the USCCB site
#[derive(Default)]
pub struct WebClient {
    client: Client,
}

impl WebClient {
    pub async fn get_for_date_id(&self, date_id: DateId) -> Result<Lectionary, Box<dyn Error>> {
        let url = Self::url_for_date(&date_id);
        debug!("Sending GET request to {}", url);
        let response = self.client.get(url).send().await?.error_for_status()?;
        let response_text = response.text().await?;
        let document = Html::parse_document(&response_text);
        Lectionary::create_from_html(date_id, document)
    }

    fn url_for_date(date_id: &DateId) -> Url {
        let url_string = format!("https://bible.usccb.org/bible/readings/{}.cfm", date_id);
        Url::parse(&url_string).expect("Formmatted string is valid URL")
    }
}
