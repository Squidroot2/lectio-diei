use std::{fs::File, io::Read};

use log::*;
use serde::{Deserialize, Serialize};

use crate::{error::ReadConfigError, path};

#[derive(Serialize, Deserialize)]
pub struct Config {
    database: DbConfig,
}

impl Config {
    pub fn from_file_or_default() -> Self {
        match Self::from_file() {
            Ok(config) => {
                debug!("Successfully retrieved config from file");
                config
            }
            Err(e) => {
                warn!("Failed to retrieve config from file ({}); Using default config settings", e);
                Self::default()
            }
        }
    }

    fn from_file() -> Result<Self, ReadConfigError> {
        let config = match path::create_and_get_config_path() {
            Ok(path) => {
                debug!("Reading config from path: {}", path.to_string_lossy());
                let mut config_string = String::new();
                File::open(path)?.read_to_string(&mut config_string)?;
                toml::from_str(&config_string)?
            }
            Err(e) => return Err(e.into()),
        };
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database: DbConfig::default(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct DbConfig {
    futures_entries: u32,
    past_entries: u32,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            futures_entries: 30,
            past_entries: 0,
        }
    }
}
