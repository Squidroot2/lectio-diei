use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io;

use log::*;
use simplelog::{
    format_description, ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter, LevelPadding, SharedLogger, TermLogger, TerminalMode,
    WriteLogger,
};

use crate::{
    colorful_logger::{ColorConfig, ColorfulLogger},
    error::PathError,
    path,
};

/// Initializes a combined logger included a terminal logger and a file logger. If file logger fails to be created, still initializes the terminal logger
pub fn init_logger() {
    let mut loggers: Vec<Box<dyn SharedLogger>> = Vec::new();
    loggers.push(color_logger());
    match file_logger() {
        Ok(file_logger) => {
            loggers.push(file_logger);
            init_combined(loggers);
        }
        Err(e) => {
            init_combined(loggers);
            error!("Failed to initialize file log: {}", e)
        }
    }
}

/// Tries to initialize the given loggers into a combined logger
fn init_combined(loggers: Vec<Box<dyn SharedLogger>>) {
    if CombinedLogger::init(loggers).is_err() {
        error!("Tried to initialize logger after already initialized")
    }
}

/// Creates an uninitialized terminal logger
fn _terminal_logger() -> Box<TermLogger> {
    TermLogger::new(
        LevelFilter::Warn,
        ConfigBuilder::new()
            .add_filter_allow_str(env!("CARGO_CRATE_NAME"))
            .set_target_level(LevelFilter::Off)
            .set_time_level(LevelFilter::Off)
            .set_thread_level(LevelFilter::Off)
            .build(),
        TerminalMode::Stderr,
        ColorChoice::Auto,
    )
}

fn color_logger() -> Box<ColorfulLogger> {
    ColorfulLogger::new(LevelFilter::Warn, ColorConfig::default())
}

/// Creates an uninitialized file logger
fn file_logger() -> Result<Box<WriteLogger<File>>, FileLoggerError> {
    let path = path::create_and_get_log_path()?;
    let file = File::options().create(true).append(true).open(path)?;
    Ok(WriteLogger::new(
        LevelFilter::Debug,
        ConfigBuilder::new()
            .set_time_format_custom(format_description!(
                "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z"
            ))
            .set_target_level(LevelFilter::Off)
            .set_level_padding(LevelPadding::Right)
            .set_thread_level(LevelFilter::Error)
            .add_filter_allow_str(env!("CARGO_CRATE_NAME"))
            .build(),
        file,
    ))
}

/// Represents a failure to open a file for the purpose of writing logs to it
#[derive(Debug)]
enum FileLoggerError {
    PathError(PathError),
    FileOpenError(io::Error),
}

impl fmt::Display for FileLoggerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PathError(e) => write!(f, "Failed to get path to log: {}", e),
            Self::FileOpenError(e) => write!(f, "Failed to open log file: {}", e),
        }
    }
}

impl From<io::Error> for FileLoggerError {
    fn from(e: io::Error) -> FileLoggerError {
        FileLoggerError::FileOpenError(e)
    }
}
impl From<PathError> for FileLoggerError {
    fn from(e: PathError) -> FileLoggerError {
        FileLoggerError::PathError(e)
    }
}

impl Error for FileLoggerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::FileOpenError(e) => Some(e),
            Self::PathError(e) => Some(e),
        }
    }
}
