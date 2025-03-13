use anyhow::Result;
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};
use walkdir::WalkDir;

use crate::config::Config;

const CONFIG_PATH: &str = "/etc/eymate/";
const DATA_PATH: &str = "/usr/share/eymate/";

fn set_permissions_recursively(path: &PathBuf, mode: u32) -> Result<()> {
    let permissions = fs::Permissions::from_mode(mode);

    for entry in WalkDir::new(path) {
        let entry = entry?;
        fs::set_permissions(entry.path(), permissions.clone())?;
    }

    Ok(())
}
pub fn get_config_dir() -> PathBuf {
    PathBuf::from(CONFIG_PATH)
}

#[allow(dead_code)]
pub fn create_config_dir() -> Result<PathBuf> {
    let config_path = get_config_dir();

    if !config_path.exists() {
        fs::create_dir_all(&config_path)?;
    }
    set_permissions_recursively(&config_path, 0o755)?;
    // fs::set_permissions(&config_path, fs::Permissions::from_mode(0o755))?;

    Ok(config_path)
}

pub fn get_data_dir() -> PathBuf {
    PathBuf::from(DATA_PATH)
}

#[allow(dead_code)]
pub fn create_data_dir() -> Result<PathBuf> {
    let data_path = get_data_dir();

    let full_path = data_path.join("users");

    if !full_path.exists() {
        fs::create_dir_all(&full_path)?;
    }

    set_permissions_recursively(&data_path, 0o755)?;
    // fs::set_permissions(&data_path, fs::Permissions::from_mode(0o755))?;

    Ok(data_path)
}

pub fn get_config_file() -> Result<PathBuf> {
    let config_file = get_config_dir().join("config.toml");

    if !config_file.exists() {
        let default_config = Config::default();
        fs::write(&config_file, toml::to_string_pretty(&default_config)?)?;
    }

    Ok(config_file)
}
