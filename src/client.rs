use log::*;
use reqwest::{Client, Url};
use scraper::Html;

use crate::date::DateId;
use crate::error::WebGetError;
use crate::html;
use crate::lectionary::Lectionary;

const BASE_URL: &str = "https://bible.usccb.org";

/// Client for interacting with the USCCB site
#[derive(Default, Clone)]
pub struct WebClient {
    client: Client,
}

impl WebClient {
    pub async fn get_for_date_id(&self, date_id: DateId) -> Result<Lectionary, WebGetError> {
        let url = Self::url_for_date(&date_id);
        let document = self.get_document_from_url(url).await?;

        if let Some(endpoint) = html::get_holiday_day_reading_link(&document) {
            info!("{date_id} seems to be a holiday. Using the link for the daytime reading");
            let url = Self::url_for_endpoint(endpoint);
            let document = self.get_document_from_url(url).await?;
            return Lectionary::create_from_html(date_id, &document).map_err(WebGetError::ParseError)
        }

        Lectionary::create_from_html(date_id, &document).map_err(WebGetError::ParseError)
    }

    async fn get_document_from_url(&self, url: Url) -> Result<Html, WebGetError> {
        debug!("Sending GET request to {}", url);
        let response = self.client.get(url).send().await.map_err(WebGetError::ClientError)?;
        if !response.status().is_success() {
            return Err(WebGetError::ErrorStatus(response.status()));
        }

        let response_text = response.text().await.map_err(WebGetError::ResponseError)?;
        Ok(Html::parse_document(&response_text))
    }

    fn url_for_date(date_id: &DateId) -> Url {
        let url_string = format!("{BASE_URL}/bible/readings/{date_id}.cfm");
        Url::parse(&url_string).expect("Formmatted string is valid URL")
    }

    fn url_for_endpoint(endpoint: &str) -> Url {
        let mut url_string = String::new();
        url_string.push_str(BASE_URL);
        url_string.push_str(endpoint);
        Url::parse(&url_string).expect("Base URL plus endpoint must be valid URL")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_url_for_date() {
        let date_id = DateId::checked_from_str("072024").unwrap();
        let url = WebClient::url_for_date(&date_id);
        assert_eq!(url.origin().ascii_serialization(), BASE_URL);
        assert_eq!(url.path(), "/bible/readings/072024.cfm");
    }

    #[test]
    fn correct_url_for_endpoint() {
        let url = WebClient::url_for_endpoint("/example/endpoint");
        assert_eq!(url.origin().ascii_serialization(), BASE_URL);
        assert_eq!(url.path(), "/example/endpoint");
    }
}
