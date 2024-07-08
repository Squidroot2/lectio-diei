//TODO Windows support
use std::env::{self, VarError};
use std::fs;
use std::path::PathBuf;

use crate::error::PathError;

/// Returns the path of the db file, after ensuring all parent directories have been created
pub fn create_and_get_db_path() -> Result<PathBuf, PathError> {
    let mut path = get_home_path().map_err(PathError::NoHome)?;
    path.push(".local");
    path.push("share");
    path.push(env!("CARGO_PKG_NAME"));
    path.push(concat!(env!("CARGO_PKG_NAME"), ".log"));

    fs::create_dir_all(path.parent().expect("Created path must have parent")).map_err(PathError::PathCreateFailure)?;

    Ok(path)
}

/// Returns the path of the log file, after ensuring all parent directories have been created
pub fn create_and_get_log_path() -> Result<PathBuf, PathError> {
    let mut path = get_home_path().map_err(PathError::NoHome)?;
    path.push(".local");
    path.push("state");
    path.push(env!("CARGO_PKG_NAME"));
    path.push(concat!(env!("CARGO_PKG_NAME"), ".log"));

    fs::create_dir_all(path.parent().expect("Created path must have parent")).map_err(PathError::PathCreateFailure)?;

    Ok(path)
}

/// Returns the path of the config file, after ensuring all parent directories have been created
pub fn create_and_get_config_path() -> Result<PathBuf, PathError> {
    let mut path = get_home_path().map_err(PathError::NoHome)?;
    path.push(".config");
    path.push(env!("CARGO_PKG_NAME"));
    path.push("config.toml");

    fs::create_dir_all(path.parent().expect("Created path must have parent")).map_err(PathError::PathCreateFailure)?;

    Ok(path)
}

fn get_home_path() -> Result<PathBuf, VarError> {
    Ok(PathBuf::from(env::var("HOME")?))
}
