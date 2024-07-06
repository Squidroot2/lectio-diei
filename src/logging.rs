use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io;

use log::SetLoggerError;
use simplelog::{format_description, ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter, TermLogger, TerminalMode, WriteLogger};

use crate::path::{self, PathError};

pub fn init_logger() -> Result<(), InitLogError> {
    let path = path::create_and_get_log_path()?;

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Warn,
            ConfigBuilder::new().add_filter_allow_str(env!("CARGO_CRATE_NAME")).build(),
            TerminalMode::Stderr,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Debug,
            ConfigBuilder::new()
                .set_time_format_custom(format_description!(
                    "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z"
                ))
                .set_target_level(LevelFilter::Off)
                .add_filter_allow_str(env!("CARGO_CRATE_NAME"))
                .build(),
            File::options().create(true).append(true).open(path)?,
        ),
    ])?;

    Ok(())
}

#[derive(Debug)]
pub enum InitLogError {
    LogError(SetLoggerError),
    PathError(PathError),
    FileCreateError(io::Error),
}

impl fmt::Display for InitLogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LogError(e) => write!(f, "Failed to initialize log: {}", e),
            Self::PathError(e) => write!(f, "Failed to get path to log: {}", e),
            Self::FileCreateError(e) => write!(f, "Failed to create log file: {}", e),
        }
    }
}

impl From<SetLoggerError> for InitLogError {
    fn from(e: SetLoggerError) -> InitLogError {
        InitLogError::LogError(e)
    }
}
impl From<io::Error> for InitLogError {
    fn from(e: io::Error) -> InitLogError {
        InitLogError::FileCreateError(e)
    }
}
impl From<PathError> for InitLogError {
    fn from(e: PathError) -> InitLogError {
        InitLogError::PathError(e)
    }
}

impl Error for InitLogError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LogError(e) => Some(e),
            Self::FileCreateError(e) => Some(e),
            Self::PathError(e) => Some(e),
        }
    }
}
