use std::fs::File;
use std::io;

use log::*;
use simplelog::{
    format_description, ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter, LevelPadding, SharedLogger, TermLogger, TerminalMode,
    ThreadPadding, WriteLogger,
};

use crate::{
    colorful_logger::{ColorConfig, ColorfulLogger},
    path::{self, PathError},
};

/// Initializes a combined logger included a terminal logger and a file logger. If file logger fails to be created, still initializes the terminal logger
pub fn init_logger(options: LoggingOptions) {
    let mut loggers: Vec<Box<dyn SharedLogger>> = Vec::new();
    loggers.push(color_logger(options));
    match file_logger() {
        Ok(file_logger) => {
            loggers.push(file_logger);
            init_combined(loggers);
        }
        Err(e) => {
            init_combined(loggers);
            error!("Failed to initialize file log: {e}");
        }
    }
}

/// Tries to initialize the given loggers into a combined logger
fn init_combined(loggers: Vec<Box<dyn SharedLogger>>) {
    if CombinedLogger::init(loggers).is_err() {
        error!("Tried to initialize logger after already initialized");
    }
}

/// Creates an uninitialized terminal logger
///
/// Unused but keeping it around for reference
// Reason: CombinedLogger::init needs boxed values
#[allow(clippy::unnecessary_box_returns)]
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

// Reason: CombinedLogger::init needs boxed values
#[allow(clippy::unnecessary_box_returns)]
fn color_logger(options: LoggingOptions) -> Box<ColorfulLogger> {
    let color_config = if options.no_color {
        ColorConfig::no_color()
    } else {
        ColorConfig::default()
    };
    ColorfulLogger::new(LevelFilter::Warn, color_config)
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
            .set_thread_padding(ThreadPadding::Left(2))
            .add_filter_allow_str(env!("CARGO_CRATE_NAME"))
            .build(),
        file,
    ))
}

#[derive(Copy, Clone)]
pub struct LoggingOptions {
    pub no_color: bool,
}

/// Represents a failure to open a file for the purpose of writing logs to it
#[derive(thiserror::Error, Debug)]
enum FileLoggerError {
    #[error("Failed to get path to log: ({0})")]
    PathError(#[from] PathError),
    #[error("Failed to open log file: ({0})")]
    FileOpenError(#[from] io::Error),
}
