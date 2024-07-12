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
    pub display: DisplayConfig,
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
                    error!("Failed to retrieve config from file: {e}; Proceeding with default config settings",);
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
        // This seems silly but the 'toml' crate is nicer to work with when using serde but only the 'toml_edit' crate supports adding comments
        let basic_toml_string = toml::to_string(&Self::default()).unwrap();
        let mut doc = basic_toml_string.parse::<DocumentMut>().unwrap();

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
    /// Will panice if key doesn't exist. Should only be used by `default_document()` which is predictable and unit tested
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
    pub future_entries: u32,
    pub past_entries: u32,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            future_entries: 30,
            past_entries: 0,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct DisplayConfig {
    pub reading_order: Vec<ReadingArg>,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            reading_order: vec![ReadingArg::Reading1, ReadingArg::Reading2, ReadingArg::Gospel],
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
