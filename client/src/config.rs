use anyhow::{Result, anyhow};
use std::{fs, path::PathBuf};
use toml::Table;

const APP_NAME: &str = "krypt";
const CONFIG_FILE: &str = "config.toml";

fn config_path() -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .ok_or_else(|| anyhow!("Could not find config directory"))?
        .join(APP_NAME);

    fs::create_dir_all(&dir)?;
    Ok(dir.join(CONFIG_FILE))
}

pub fn get_value(table: &str, key: &str) -> Result<Option<String>> {
    let path = config_path()?;

    let content = fs::read_to_string(&path).map_err(|_| anyhow!("Could not read config file"))?;
    let mut toml = content
        .parse::<toml::Table>()
        .map_err(|_| anyhow!("Corrupt config file"))?;

    let toml = if table.is_empty() {
        &mut toml
    } else {
        match toml.get_mut(table) {
            Some(table) => table.as_table_mut().unwrap(),
            None => {
                toml.insert(table.to_owned(), toml::Value::Table(Table::new()));

                toml.get_mut(table)
                    .unwrap()
                    .as_table_mut()
                    .unwrap()
            }
        }
    };

    Ok(toml.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string()))
}

pub fn add_entry(table: &str, key: &str, value: &str) -> Result<Option<String>> {
    let path = config_path()?;

    let content = fs::read_to_string(&path).map_err(|_| anyhow!("Could not read config file"))?;

    let mut toml = content
        .parse::<toml::Table>()
        .map_err(|_| anyhow!("Corrupt config file"))?;

    let old = if table.is_empty() {
        &mut toml
    } else {
        match toml.get_mut(table) {
            Some(table) => table.as_table_mut().unwrap(),
            None => {
                toml.insert(table.to_owned(), toml::Value::Table(Table::new()));

                toml.get_mut(table)
                    .ok_or_else(|| anyhow!("Table not found"))?
                    .as_table_mut()
                    .unwrap()
            }
        }
    }
    .insert(key.to_string(), toml::value::Value::String(value.to_string()))
    .map(|v| v.as_str().unwrap().to_string());

    fs::write(&path, toml.to_string())?;
    Ok(old)
}

pub fn clear_value(table: &str, key: &str) -> Result<()> {
    let path = config_path()?;

    let content = fs::read_to_string(&path).map_err(|_| anyhow!("Could not read config file"))?;

    let mut toml = content
        .parse::<toml::Table>()
        .map_err(|_| anyhow!("Corrupt config file"))?;

    let toml = if table.is_empty() {
        &mut toml
    } else {
        match toml.get_mut(table) {
            Some(table) => table.as_table_mut().unwrap(),
            None => {
                toml.insert(table.to_owned(), toml::Value::Table(Table::new()));

                toml.get_mut(table)
                    .ok_or_else(|| anyhow!("Table not found"))?
                    .as_table_mut()
                    .unwrap()
            }
        }
    };

    toml.remove(key)
        .ok_or_else(|| anyhow!("Username was not set"))?;

    fs::write(&path, table.to_string())?;
    Ok(())
}
