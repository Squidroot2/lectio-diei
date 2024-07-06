use std::error::Error;
use std::fmt::{self, Display};

use log::debug;
use reqwest::{Client, StatusCode, Url};
use scraper::Html;

use crate::date::DateId;
use crate::lectionary::{Lectionary, LectionaryHtmlError};

/// Client for interacting with the USCCB site
#[derive(Default)]
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
        Lectionary::create_from_html(date_id, document).map_err(WebGetError::ParseError)
    }

    fn url_for_date(date_id: &DateId) -> Url {
        let url_string = format!("https://bible.usccb.org/bible/readings/{}.cfm", date_id);
        Url::parse(&url_string).expect("Formmatted string is valid URL")
    }
}

#[derive(Debug)]
pub enum WebGetError {
    ClientError(reqwest::Error),
    ErrorStatus(StatusCode),
    ResponseError(reqwest::Error),
    ParseError(LectionaryHtmlError),
}

impl Display for WebGetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ClientError(e) => write!(f, "Web client error on GET request: {}", e),
            Self::ErrorStatus(code) => write!(f, "Error status code on GET request: {}", code),
            Self::ResponseError(e) => write!(f, "Error reading response: {}", e),
            Self::ParseError(e) => write!(f, "Error creating lectionary from html: {}", e),
        }
    }
}

impl Error for WebGetError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ClientError(e) => Some(e),
            Self::ErrorStatus(_) => None,
            Self::ResponseError(e) => Some(e),
            Self::ParseError(e) => Some(e),
        }
    }
}
