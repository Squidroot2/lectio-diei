use std::{
    env,
    fs::File,
    io::{self, Read, Write},
    path::PathBuf,
};

use clap::ValueEnum;
use log::*;
use serde::{Deserialize, Serialize};
use toml::{de, ser::ValueSerializer};
use toml_edit::{self, DocumentMut};

use crate::{
    args::ReadingArg,
    error::{InitConfigError, ReadConfigError},
    path,
};

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub database: DbConfig,
}

impl Config {
    pub fn from_file_or_default() -> Self {
        match path::create_and_get_config_path() {
            Ok(path) => match Self::from_file(&path) {
                Ok(config) => {
                    info!("Successfully retrieved config from '{}'", path.to_string_lossy());
                    config
                }
                Err(ReadConfigError::NotFound(_)) => {
                    warn!(
                        "No config file found at '{}'; Generating new config file with default settings",
                        path.to_string_lossy()
                    );
                    // Going to create a config with force=true because if someone created a file in the clock cycles between failing to
                    //  retrieve and creating, they deserve to lose their config tbh
                    if let Err(create_error) = Self::create_config(&path, true) {
                        error!("Failed to create config file ({create_error})");
                    }
                    Self::default()
                }
                Err(e) => {
                    error!("Failed to retrieve config from file ({e}); Proceeding with default config settings",);
                    Self::default()
                }
            },
            Err(e) => {
                error!("Failed to determine path for config file ({e}); Proceeding with default config settings",);
                Self::default()
            }
        }
    }

    pub fn initialize_default_config(force: bool) -> Result<(), InitConfigError> {
        debug!("Creating a default config with force={}", force);
        match path::create_and_get_config_path() {
            Ok(path) => Self::create_config(&path, force).map_err(InitConfigError::from),
            Err(e) => Err(e.into()),
        }
    }

    //TODO This returns a ReadConfigError even when the error is a write error
    pub fn upgrade_config() -> Result<(), ReadConfigError> {
        let path = path::create_and_get_config_path()?;
        //TODO handle case with no config file
        let config = match Self::from_file(&path) {
            Ok(config) => config,
            Err(ReadConfigError::NotFound(_)) => {
                warn!(
                    "Tried to upgrade missing config file at '{}'. Creating new config instead",
                    path.to_string_lossy()
                );
                Self::default()
            }
            Err(e) => {
                error!("Error while trying to read config at '{}'", path.to_string_lossy());
                return Err(e);
            }
        };
        let commented_doc = config.to_commented_doc();
        File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?
            .write_all(commented_doc.to_string().as_bytes())?;
        debug!("Wrote upgraded config to '{}'", path.to_string_lossy());
        Ok(())
    }

    fn from_file(path: &PathBuf) -> Result<Self, ReadConfigError> {
        debug!("Reading config from path: {}", path.to_string_lossy());
        let mut config_string = String::new();
        File::open(path)?.read_to_string(&mut config_string)?;
        let config = de::from_str(&config_string)?;
        Ok(config)
    }

    /// Writes the default config to the given path
    ///
    /// If force is false, fails if file already exists at path, otherwise overwrites file.
    /// Also fails on other IO error (permissions etc.)
    fn create_config(path: &PathBuf, force: bool) -> Result<(), io::Error> {
        let config_string = Self::default_document().to_string();
        let mut file = if force { File::create(path)? } else { File::create_new(path)? };
        file.write_all(config_string.as_bytes())
    }

    /// Generates a default config document including comments
    fn default_document() -> DocumentMut {
        Self::default().to_commented_doc()
    }

    fn to_commented_doc(&self) -> DocumentMut {
        let basic_toml_string = toml::to_string(&self).expect("Default config should be serialiable to TOML string");
        let mut doc = basic_toml_string
            .parse::<DocumentMut>()
            .expect("Serialized config string should be parseable to TOML document");

        // Adds a header comment
        doc.decor_mut()
            .set_prefix(format!("# GENERATED ON VERSION: {}\n\n", env!("CARGO_PKG_VERSION")));

        Self::set_key_comment(
            &mut doc,
            "display",
            "reading_order",
            &format!(
                "Defines which readings and what order. Possible values: {}\n# Use empty array to only display day",
                ReadingArg::variant_string()
            ),
        );

        Self::set_key_comment(
            &mut doc,
            "display",
            "original_linebreaks",
            "Whether to use original linebreaks as displayed on USCCB site. If true, max_width is ignored. Note: Resp. Psalm always uses original line breaks",
        );

        Self::set_key_comment(
            &mut doc,
            "display" ,
            "max_width" ,
            "Maximum width for formatting readings. Ignored if original_linebreaks is true. Not used for Psalm. Set to 0 for no line breaks" );

        Self::set_key_comment(
            &mut doc,
            "database",
            "future_entries",
            "Number of days in to the future to try to keep in the database. Includes today (i.e. a value of 1 will only store today's readings)",
        );
        Self::set_key_comment(
            &mut doc,
            "database",
            "past_entries",
            "Number of days in to the past to try to keep in the database",
        );

        doc
    }

    /// Puts a comment above a key.
    ///
    /// Will panic if key doesn't exist.
    fn set_key_comment(doc: &mut DocumentMut, table: &str, key: &str, comment: &str) {
        let formatted = format!("# {comment}\n");
        doc.get_mut(table)
            .unwrap()
            .as_table_mut()
            .unwrap()
            .key_mut(key)
            .unwrap()
            .leaf_decor_mut()
            .set_prefix(formatted);
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DbConfig {
    #[serde(default = "DbConfig::default_future_entries")]
    pub future_entries: u32,
    #[serde(default)]
    pub past_entries: u32,
}

impl DbConfig {
    fn default_future_entries() -> u32 {
        30
    }
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            future_entries: Self::default_future_entries(),
            past_entries: u32::default(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "DisplayConfig::default_reading_order")]
    pub reading_order: Vec<ReadingArg>,
    #[serde(default)]
    pub original_linebreaks: bool,
    #[serde(default = "DisplayConfig::default_width")]
    pub max_width: u16,
}

impl DisplayConfig {
    fn default_reading_order() -> Vec<ReadingArg> {
        vec![ReadingArg::Reading1, ReadingArg::Reading2, ReadingArg::Gospel]
    }

    fn default_width() -> u16 {
        140
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            reading_order: Self::default_reading_order(),
            original_linebreaks: bool::default(),
            max_width: Self::default_width(),
        }
    }
}

impl ReadingArg {
    /// Returns a string that represents all of the variants joined by commas
    ///
    /// Used for displaying a comment showing the possible options
    fn variant_string() -> String {
        let mut names = Vec::new();
        for variant in Self::value_variants() {
            let mut name_buffer = String::new();
            let serializer = ValueSerializer::new(&mut name_buffer);
            Serialize::serialize(variant, serializer).unwrap();
            names.push(name_buffer);
        }

        names.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ensures that the default config serializes to a valid document
    #[test]
    fn default_document_serializes() {
        // Just make sure it doesn't panic
        let _ = Config::default_document();
    }
}
