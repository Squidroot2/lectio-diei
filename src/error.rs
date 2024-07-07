use std::{
    env::VarError,
    error::Error,
    fmt::{self, Display},
    io,
};

use chrono::ParseError;
use reqwest::StatusCode;
use sqlx::migrate::MigrateError;
use toml::de;

use crate::lectionary::ReadingName;

#[derive(Debug)]
pub enum ApplicationError {
    NotImplemented,
    BadArgument(ArgumentError),
    DatabaseError(DatabaseError),
    RetrievalError(RetrievalError),
}

impl ApplicationError {
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::BadArgument(_) => 3,
            Self::DatabaseError(_) => 4,
            Self::RetrievalError(_) => 5,
            Self::NotImplemented => 100,
        }
    }
}

impl Display for ApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadArgument(e) => write!(f, "Bad arugment: {}", e),
            Self::DatabaseError(e) => write!(f, "Fatal database error: {}", e),
            Self::RetrievalError(e) => write!(f, "Can't display lectionary: {}", e),
            Self::NotImplemented => write!(f, "Functionality Not Implemented"),
        }
    }
}

impl Error for ApplicationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::BadArgument(e) => Some(e),
            Self::DatabaseError(e) => Some(e),
            Self::RetrievalError(e) => Some(e),
            Self::NotImplemented => None,
        }
    }
}

impl From<DatabaseInitError> for ApplicationError {
    fn from(value: DatabaseInitError) -> Self {
        Self::from(DatabaseError::InitError(value))
    }
}

impl From<DatabaseError> for ApplicationError {
    fn from(value: DatabaseError) -> Self {
        Self::DatabaseError(value)
    }
}

impl From<ArgumentError> for ApplicationError {
    fn from(value: ArgumentError) -> Self {
        Self::BadArgument(value)
    }
}

impl From<RetrievalError> for ApplicationError {
    fn from(value: RetrievalError) -> Self {
        Self::RetrievalError(value)
    }
}

#[derive(Debug)]
pub enum ArgumentError {
    InvalidDate(ParseError),
}

impl Display for ArgumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDate(e) => write!(f, "Invalid date Argument: {}", e),
        }
    }
}

impl Error for ArgumentError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidDate(e) => Some(e),
        }
    }
}

/// A failure to retrieve a lectionary from the database, web, or both
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

#[derive(Debug)]
pub enum DatabaseError {
    InitError(DatabaseInitError),
    GetError(DatabaseGetError),
}

impl Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InitError(e) => write!(f, "{}", e),
            Self::GetError(e) => write!(f, "{}", e),
        }
    }
}

impl Error for DatabaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InitError(e) => Some(e),
            Self::GetError(e) => Some(e),
        }
    }
}

impl From<DatabaseInitError> for DatabaseError {
    fn from(value: DatabaseInitError) -> Self {
        Self::InitError(value)
    }
}

impl From<DatabaseGetError> for DatabaseError {
    fn from(value: DatabaseGetError) -> Self {
        Self::GetError(value)
    }
}

#[derive(Debug)]
pub enum DatabaseGetError {
    NotPresent,
    QueryError(sqlx::Error),
}

impl Display for DatabaseGetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotPresent => write!(f, "Query returned no results"),
            Self::QueryError(e) => write!(f, "Select Query failed: {}", e),
        }
    }
}

impl Error for DatabaseGetError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NotPresent => None,
            Self::QueryError(e) => Some(e),
        }
    }
}

impl From<sqlx::Error> for DatabaseGetError {
    fn from(value: sqlx::Error) -> Self {
        Self::QueryError(value)
    }
}

#[derive(Debug)]
pub enum DatabaseInitError {
    CannotGetUrl(PathError),
    CreateDatabaseError(sqlx::Error),
    PoolCreationFailed(sqlx::Error),
    PragmaForeignKeysFailure(sqlx::Error),
    MigrationError(MigrateError),
}

impl Display for DatabaseInitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CannotGetUrl(e) => write!(f, "Cannot construct database URL: {}", e),
            Self::CreateDatabaseError(e) => write!(f, "Cannot create database: {}", e),
            Self::PoolCreationFailed(e) => write!(f, "Failed to create a connection pool for the database: {}", e),
            Self::PragmaForeignKeysFailure(e) => write!(f, "Failed to enable foreign keys in the database: {}", e),
            Self::MigrationError(e) => write!(f, "Failed to run migration scripts for database: {}", e),
        }
    }
}

impl Error for DatabaseInitError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CannotGetUrl(e) => Some(e),
            Self::CreateDatabaseError(e) => Some(e),
            Self::PoolCreationFailed(e) => Some(e),
            Self::PragmaForeignKeysFailure(e) => Some(e),
            Self::MigrationError(e) => Some(e),
        }
    }
}

/// Represents a failure to parse a HTML document in to a Lectionary struct
#[derive(Debug)]
pub enum LectionaryHtmlError {
    NoContainerFound,
    NoDayNameElementFound,
    MissingReading(ReadingName),
}

impl Display for LectionaryHtmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoContainerFound => write!(f, "No main readings container found"),
            Self::NoDayNameElementFound => write!(f, "No day name element found"),
            Self::MissingReading(name) => write!(f, "Missing required reading: {}", name),
        }
    }
}

impl Error for LectionaryHtmlError {}

/// Represents a failure to parse an HTML element into a Reading struct
#[derive(Debug)]
pub enum ReadingHtmlError {
    MissingLocation,
    MissingContent,
}

impl Display for ReadingHtmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingLocation => write!(f, "Missing Location"),
            Self::MissingContent => write!(f, "Missing Content"),
        }
    }
}

impl Error for ReadingHtmlError {}

/// Error for TryFrom\<String> on ReadingName
#[derive(Debug)]
pub struct ReadingNameFromStringError {
    value: String,
}
impl std::error::Error for ReadingNameFromStringError {}
impl fmt::Display for ReadingNameFromStringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid reading name: {}", self.value)
    }
}
impl From<String> for ReadingNameFromStringError {
    fn from(value: String) -> Self {
        Self { value }
    }
}

/// Represents a failure to read the config file
#[derive(Debug)]
pub enum ReadConfigError {
    CannotGetPath(PathError),
    CannotOpenFile(io::Error),
    DeserializationError(de::Error),
}

impl fmt::Display for ReadConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CannotGetPath(e) => write!(f, "Cannot get path to config file: {}", e),
            Self::CannotOpenFile(e) => write!(f, "Cannot read config file: {}", e),
            Self::DeserializationError(e) => write!(f, "Failed to deserialize config file: {}", e),
        }
    }
}
impl Error for ReadConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CannotGetPath(e) => Some(e),
            Self::CannotOpenFile(e) => Some(e),
            Self::DeserializationError(e) => Some(e),
        }
    }
}
impl From<PathError> for ReadConfigError {
    fn from(value: PathError) -> Self {
        Self::CannotGetPath(value)
    }
}
impl From<io::Error> for ReadConfigError {
    fn from(value: io::Error) -> Self {
        Self::CannotOpenFile(value)
    }
}
impl From<de::Error> for ReadConfigError {
    fn from(value: de::Error) -> Self {
        Self::DeserializationError(value)
    }
}
/// Represents a failure to identify a file path
#[derive(Debug)]
pub enum PathError {
    NoHome(VarError),
    PathCreateFailure(io::Error),
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoHome(e) => write!(f, "Could not get HOME environment variable: {}", e),
            Self::PathCreateFailure(e) => write!(f, "Failed to create parent directory: {}", e),
        }
    }
}

impl Error for PathError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NoHome(e) => Some(e),
            Self::PathCreateFailure(e) => Some(e),
        }
    }
}
