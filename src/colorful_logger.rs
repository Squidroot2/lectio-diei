//! Module for my custom logger, the ColorfulLogger

// Module heavily based on the termlog module from the simplelog crate.
// LICENSE for simplelog:
// The MIT License (MIT)
//
// Copyright (c) 2015 Victor Brekenfeld
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use log::{set_boxed_logger, set_max_level, Level, LevelFilter, Log, Metadata, Record, SetLoggerError};
use std::io::{Error, Write};
use std::sync::Mutex;
use termcolor::{BufferedStandardStream, Color, ColorChoice};
use termcolor::{ColorSpec, WriteColor};

use simplelog::{Config, SharedLogger};

const ERROR_RED: Color = Color::Rgb(225, 60, 45);
const WARNING_YELLOW: Color = Color::Rgb(250, 190, 75);

/// Defines the Colors for each log level
///
/// Since the properties of the simplelog::Config are private to the the crate, we can't use it. Instead we are using a custom config struct
pub struct ColorConfig {
    error_color: Option<Color>,
    warn_color: Option<Color>,
    info_color: Option<Color>,
    debug_color: Option<Color>,
    trace_color: Option<Color>,
}

impl ColorConfig {
    /// Disables color. Useful when being redirected to a file
    pub fn no_color() -> Self {
        Self {
            error_color: None,
            warn_color: None,
            info_color: None,
            debug_color: None,
            trace_color: None,
        }
    }

    /// Gets the color for the given level
    fn for_level(&self, level: Level) -> Option<Color> {
        match level {
            Level::Error => self.error_color,
            Level::Warn => self.warn_color,
            Level::Info => self.info_color,
            Level::Debug => self.debug_color,
            Level::Trace => self.trace_color,
        }
    }
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            error_color: Some(ERROR_RED),
            warn_color: Some(WARNING_YELLOW),
            info_color: Some(Color::Green),
            debug_color: Some(Color::White),
            trace_color: Some(Color::Black),
        }
    }
}

/// The ColorfulLogger struct. Provides a stderr based, colorful Logger implementation.
///
/// Like the TermLogger from simplelog but colors the entire line rather than just the level
pub struct ColorfulLogger {
    level: LevelFilter,
    config: ColorConfig,
    stream: Mutex<BufferedStandardStream>,
}

impl ColorfulLogger {
    /// Globally initializes the `ColorfulLogger` as the one and only used log facility.
    ///
    /// Takes the desired `Level` and `ColorConfig` as arguments. They cannot be changed later on.
    /// Fails if another Logger was already initialized
    ///
    /// Not currently used, but keeping it around since other shared loggers have an init function
    pub fn _init(log_level: LevelFilter, color_config: ColorConfig) -> Result<(), SetLoggerError> {
        let logger = ColorfulLogger::new(log_level, color_config);
        set_max_level(log_level);
        set_boxed_logger(logger)?;
        Ok(())
    }

    /// Allows to create a new logger, that can be independently used, no matter whats globally set.
    /// Intended for use in a combined logger
    ///
    /// Takes the desired `Level` and `ColorConfig` as arguments. They cannot be changed later on.
    ///
    /// Returns a `Box`ed Colorful Logger
    pub fn new(log_level: LevelFilter, config: ColorConfig) -> Box<ColorfulLogger> {
        let stream = BufferedStandardStream::stderr(ColorChoice::Always);

        Box::new(ColorfulLogger {
            level: log_level,
            config,
            stream: Mutex::new(stream),
        })
    }

    fn try_log_term(&self, record: &Record<'_>, terminal_stream: &mut BufferedStandardStream) -> Result<(), Error> {
        let color = self.config.for_level(record.level());

        // Ignore error
        terminal_stream.set_color(ColorSpec::new().set_fg(color))?;

        // On error, record level
        if record.level() == Level::Error {
            write!(terminal_stream, "{}: ", record.level())?;
        };

        writeln!(terminal_stream, "{}", record.args())?;

        // Ignore error
        terminal_stream.reset()?;

        // The log crate holds the logger as a `static mut`, which isn't dropped
        // at program exit: https://doc.rust-lang.org/reference/items/static-items.html
        // Sadly, this means we can't rely on the BufferedStandardStreams flushing
        // themselves on the way out, so to avoid the Case of the Missing 8k,
        // flush each entry.
        terminal_stream.flush()
    }

    fn try_log(&self, record: &Record<'_>) -> Result<(), Error> {
        if self.enabled(record.metadata()) {
            let mut stream = self.stream.lock().unwrap();

            self.try_log_term(record, &mut stream)
        } else {
            Ok(())
        }
    }
}

impl Log for ColorfulLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record<'_>) {
        let _ = self.try_log(record);
    }

    fn flush(&self) {
        let mut stream = self.stream.lock().unwrap();
        let _ = stream.flush();
    }
}

impl SharedLogger for ColorfulLogger {
    fn level(&self) -> LevelFilter {
        self.level
    }

    fn config(&self) -> Option<&Config> {
        None
    }

    fn as_log(self: Box<Self>) -> Box<dyn Log> {
        Box::new(*self)
    }
}
