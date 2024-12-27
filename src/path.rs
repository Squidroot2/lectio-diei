//TODO Windows support
use std::env::{self, VarError};
use std::path::PathBuf;
use std::{fs, io};

use log::*;

static DB_FILE_NAME: &str = concat!(env!("CARGO_PKG_NAME"), ".db");
static LOG_FILE_NAME: &str = concat!(env!("CARGO_PKG_NAME"), ".log");
const CONFIG_FILE_NAME: &str = "config.toml";
/// Used for logging
#[cfg(target_family = "unix")]
const STATE_ENV_VAR: &str = "XDG_STATE_HOME";
/// Used for data
const DATA_ENV_VAR: &str = "XDG_DATA_HOME";

/// Used for data and logging
#[cfg(target_family = "windows")]
const LOCALAPPDATA_ENV_VAR: &str = "LOCALAPPDATA";
/// Used for config
#[cfg(target_family = "windows")]
const APPDATA_ENV_VAR: &str = "APPDATA";
#[cfg(target_family = "unix")]
const CONFIG_ENV_VAR: &str = "XDG_CONFIG_HOME";
/// Used as an alternative when XDG paths not defined
#[cfg(target_family = "unix")]
const HOME_ENV_VAR: &str = "HOME";
//TODO Probably don't need to return errors. Just log and return an option for these public functions
/// Returns the path of the db file, after ensuring all parent directories have been created
pub fn create_and_get_db_path() -> Result<PathBuf, PathError> {
    #[cfg(target_family = "unix")]
    let mut path = get_xdg_data_home()?;

    #[cfg(target_family = "windows")]
    let mut path = get_local_app_data()?;

    path.push(env!("CARGO_PKG_NAME"));
    path.push(DB_FILE_NAME);

    fs::create_dir_all(path.parent().expect("Created path must have parent")).map_err(PathError::PathCreateFailure)?;

    Ok(path)
}

/// Returns the path of the log file, after ensuring all parent directories have been created
pub fn create_and_get_log_path() -> Result<PathBuf, PathError> {
    #[cfg(target_family = "unix")]
    let mut path = get_xdg_state_home()?;

    #[cfg(target_family = "windows")]
    let mut path = get_local_app_data()?;

    path.push(env!("CARGO_PKG_NAME"));
    path.push(LOG_FILE_NAME);

    fs::create_dir_all(path.parent().expect("Created path must have parent")).map_err(PathError::PathCreateFailure)?;

    Ok(path)
}

/// Returns the path of the config file, after ensuring all parent directories have been created
pub fn create_and_get_config_path() -> Result<PathBuf, PathError> {
    #[cfg(target_family = "unix")]
    let mut config_path = get_xdg_config_home()?;

    #[cfg(target_family = "windows")]
    let mut config_path = get_app_data()?;

    config_path.push(env!("CARGO_PKG_NAME"));
    config_path.push(CONFIG_FILE_NAME);

    fs::create_dir_all(config_path.parent().expect("Created path must have parent")).map_err(PathError::PathCreateFailure)?;

    Ok(config_path)
}

#[cfg(target_family = "windows")]
fn get_local_app_data() -> Result<PathBuf, PathError> {
    env::var(LOCALAPPDATA_ENV_VAR).map(PathBuf::from).map_err(|e| PathError::NoEnv {
        var_name: LOCALAPPDATA_ENV_VAR,
        source: e,
    })
}

#[cfg(target_family = "windows")]
fn get_app_data() -> Result<PathBuf, PathError> {
    env::var(APPDATA_ENV_VAR).map(PathBuf::from).map_err(|e| PathError::NoEnv {
        var_name: APPDATA_ENV_VAR,
        source: e,
    })
}

/// First trie `$XDG_STATE_HOME`, then tries $HOME/.local/state
fn get_xdg_state_home() -> Result<PathBuf, PathError> {
    let xdg_path = match env::var(STATE_ENV_VAR) {
        Ok(path_str) => PathBuf::from(path_str),
        Err(no_xdg_error) => {
            debug!("Failed to read environment variable '{STATE_ENV_VAR}': {no_xdg_error}");
            match get_home_path() {
                Ok(mut path) => {
                    path.push(".local");
                    path.push("state");
                    path
                }
                Err(no_home_error) => return Err(no_home_error),
            }
        }
    };
    Ok(xdg_path)
}

/// First tries `$XDG_DATA_HOME`, then tries $HOME/.local/share
fn get_xdg_data_home() -> Result<PathBuf, PathError> {
    let xdg_path = match env::var(DATA_ENV_VAR) {
        Ok(path_str) => PathBuf::from(path_str),
        Err(no_xdg_error) => {
            debug!("Failed to read environment variable '{DATA_ENV_VAR}': {no_xdg_error}");
            match get_home_path() {
                Ok(mut path) => {
                    path.push(".local");
                    path.push("share");
                    path
                }
                Err(no_home_error) => return Err(no_home_error),
            }
        }
    };
    Ok(xdg_path)
}

/// First tries `$XDG_CONFIG_HOME`, then tries $HOME/.config/
fn get_xdg_config_home() -> Result<PathBuf, PathError> {
    let xdg_path = match env::var(CONFIG_ENV_VAR) {
        Ok(path_str) => PathBuf::from(path_str),
        Err(no_xdg_error) => {
            debug!("Failed to read environment variable '{CONFIG_ENV_VAR}': {no_xdg_error}");
            match get_home_path() {
                Ok(mut path) => {
                    path.push(".config");
                    path
                }
                Err(no_home_error) => return Err(no_home_error),
            }
        }
    };
    Ok(xdg_path)
}

fn get_home_path() -> Result<PathBuf, PathError> {
    env::var(HOME_ENV_VAR).map(PathBuf::from).map_err(PathError::no_home_env)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn db_path_present() {
        let db_path = create_and_get_db_path().unwrap();
        assert!(db_path.parent().unwrap().is_dir());
        assert_eq!("db", db_path.extension().unwrap().to_string_lossy());
    }

    #[test]
    fn log_path_present() {
        let log_path = create_and_get_log_path().unwrap();
        assert!(log_path.parent().unwrap().is_dir());
        assert_eq!("log", log_path.extension().unwrap().to_string_lossy());
    }

    #[test]
    fn config_path_present() {
        let config_path = create_and_get_config_path().unwrap();
        assert!(config_path.parent().unwrap().is_dir());
        assert_eq!("toml", config_path.extension().unwrap().to_string_lossy());
    }
}

/// Represents a failure to identify a file path
#[derive(thiserror::Error, Debug)]
pub enum PathError {
    #[error("Could not get {var_name} environment variable: ({source})")]
    NoEnvVar {
        var_name: &'static str,
        #[source]
        source: VarError,
    },
    #[error("Failed to create parent directory: ({0})")]
    PathCreateFailure(#[from] io::Error),
}

impl PathError {
    fn no_home_env(error: VarError) -> Self {
        Self::NoEnvVar {
            var_name: HOME_ENV_VAR,
            source: error,
        }
    }
}
