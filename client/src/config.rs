use std::{fs, path::PathBuf};
use anyhow::{anyhow, Result};

const APP_NAME: &str = "krypt";
const CONFIG_FILE: &str = "config.toml";

fn config_path() -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .ok_or_else(|| anyhow!("Could not find config directory"))?
        .join(APP_NAME);
    
    fs::create_dir_all(&dir)?;
    Ok(dir.join(CONFIG_FILE))
}

pub fn get_username() -> Result<String> {
    let path = config_path()?;
    
    let content = fs::read_to_string(&path)
        .map_err(|_| anyhow!("Not logged in, run `krypt [signup | login]` first"))?;
    
    let table = content.parse::<toml::Table>()
        .map_err(|_| anyhow!("Corrupt config file"))?;
    
    table["username"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("Username not found in config"))
}

pub fn set_username(username: &str) -> Result<()> {
    let path = config_path()?;
    
    fs::write(path, format!("username = \"{username}\"\n"))?;
    Ok(())
}

pub fn clear_username() -> Result<()> {
    let path = config_path()?;
    
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}