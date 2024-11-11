use crate::utils;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

const PROGRAM_NAME: &str = "moree";
const DEFAULT_STATE_FILE: &str = "state.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PackageState {
    pub explicit: bool,
    pub memo: Option<String>,
}

pub fn get_state_file_path(custom_path: &Option<PathBuf>) -> io::Result<PathBuf> {
    match custom_path {
        Some(path) => get_custom_state_file_path(path),
        None => get_default_state_file_path(),
    }
}

fn get_custom_state_file_path(custom_path: &PathBuf) -> io::Result<PathBuf> {
    // Ensure the path isn't a directory
    if custom_path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The provided path is a directory, not a file.",
        ));
    }

    // Create parent directories if they don't exist
    if let Some(parent_dir) = custom_path.parent() {
        fs::create_dir_all(parent_dir)?;
    }

    // Check if the extension is .json (or allow override)
    if custom_path.extension().and_then(|ext| ext.to_str()) != Some("json") {
        println!("The provided path does not have a .json extension. Are you sure you want to use this path? (y/n)");
        let confirmed = utils::confirm_prompt_with_default(false)?;
        if !confirmed {
            return Err(io::Error::new(io::ErrorKind::Other, "Operation cancelled."));
        }
    }

    Ok(custom_path.clone())
}

fn get_default_state_file_path() -> io::Result<PathBuf> {
    let data_dir = dirs::data_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Could not determine data directory."))?
        .join(PROGRAM_NAME);

    fs::create_dir_all(&data_dir)?;
    Ok(data_dir.join(DEFAULT_STATE_FILE))
}

pub fn load_package_states(file_path: &PathBuf) -> io::Result<HashMap<String, PackageState>> {
    let data = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(HashMap::new()), // Return empty HashMap if file not found.
        Err(err) => return Err(err),
    };

    serde_json::from_str(&data).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Could not parse state file: {}", e),
        )
    })
}

pub fn save_package_states(
    file_path: &PathBuf,
    package_states: &HashMap<String, PackageState>,
) -> io::Result<()> {
    let data = serde_json::to_string_pretty(package_states)?; // Pretty print for readability
    fs::write(file_path, data)
}
