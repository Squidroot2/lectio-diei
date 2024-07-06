//TODO Windows support
use std::env::{self, VarError};
use std::fs;
use std::path::PathBuf;

use crate::error::PathError;

/// Gets the path to the db file (Linux Only). Creates directory if it does not exist
pub fn create_and_get_db_path() -> Result<PathBuf, PathError> {
    let mut path = get_home_path().map_err(PathError::NoHome)?;
    path.push(".local");
    path.push("share");
    path.push(env!("CARGO_CRATE_NAME"));
    path.push("data.db");

    fs::create_dir_all(path.parent().expect("Created path must have parent")).map_err(PathError::PathCreateFailure)?;

    Ok(path)
}

pub fn create_and_get_log_path() -> Result<PathBuf, PathError> {
    let mut path = get_home_path().map_err(PathError::NoHome)?;
    path.push(".local");
    path.push("state");
    path.push(env!("CARGO_CRATE_NAME"));
    path.push("debug.log");

    fs::create_dir_all(path.parent().expect("Created path must have parent")).map_err(PathError::PathCreateFailure)?;

    Ok(path)
}

fn get_home_path() -> Result<PathBuf, VarError> {
    Ok(PathBuf::from(env::var("HOME")?))
}
