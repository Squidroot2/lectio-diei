//TODO Windows support
use std::env::{self, VarError};
use std::path::PathBuf;
use std::{fs, io};

use log::*;

//TODO Probably don't need to return errors. Just log and return an option for these public functions
/// Returns the path of the db file, after ensuring all parent directories have been created
pub fn create_and_get_db_path() -> Result<PathBuf, PathError> {
    let mut path = get_xdg_data_home().map_err(PathError::NoHome)?;
    path.push(env!("CARGO_PKG_NAME"));
    path.push(concat!(env!("CARGO_PKG_NAME"), ".db"));

    fs::create_dir_all(path.parent().expect("Created path must have parent")).map_err(PathError::PathCreateFailure)?;

    Ok(path)
}

/// Returns the path of the log file, after ensuring all parent directories have been created
pub fn create_and_get_log_path() -> Result<PathBuf, PathError> {
    let mut path = get_xdg_state_home().map_err(PathError::NoHome)?;
    path.push(env!("CARGO_PKG_NAME"));
    path.push(concat!(env!("CARGO_PKG_NAME"), ".log"));

    fs::create_dir_all(path.parent().expect("Created path must have parent")).map_err(PathError::PathCreateFailure)?;

    Ok(path)
}

/// Returns the path of the config file, after ensuring all parent directories have been created
pub fn create_and_get_config_path() -> Result<PathBuf, PathError> {
    let mut config_path = get_xdg_config_home().map_err(PathError::NoHome)?;
    config_path.push(env!("CARGO_PKG_NAME"));
    config_path.push("config.toml");

    fs::create_dir_all(config_path.parent().expect("Created path must have parent")).map_err(PathError::PathCreateFailure)?;

    Ok(config_path)
}

/// First trie `$XDG_STATE_HOME`, then tries $HOME/.local/state
fn get_xdg_state_home() -> Result<PathBuf, VarError> {
    const STATE_ENV_VAR: &str = "XDG_STATE_HOME";
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
fn get_xdg_data_home() -> Result<PathBuf, VarError> {
    const DATA_ENV_VAR: &str = "XDG_DATA_HOME";
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
fn get_xdg_config_home() -> Result<PathBuf, VarError> {
    const CONFIG_ENV_VAR: &str = "XDG_CONFIG_HOME";
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

fn get_home_path() -> Result<PathBuf, VarError> {
    Ok(PathBuf::from(env::var("HOME")?))
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
    #[error("Could not get HOME environment variable: ({0})")]
    NoHome(#[from] VarError),
    #[error("Failed to create parent directory: ({0})")]
    PathCreateFailure(#[from] io::Error),
}
