use anyhow::{Result, anyhow};
use clap::ArgMatches;
use reqwest::StatusCode;
use shared::models::{ChangeKeyPayload, Credentials, Key, KeyPayload};

use crate::util::{ToHeader, get_client, try_or_read_password, try_or_read_username};

pub fn get_key(matches: &ArgMatches) -> Result<()> {
    let stdin = std::io::stdin();

    let username = try_or_read_username(&stdin)?;
    let password = try_or_read_password(&username, &stdin)?;

    let key = matches
        .get_one::<String>("name")
        .ok_or_else(|| anyhow!("no key name provided"))?;

    let url = env!("SERVER_URL").trim_end_matches("/").to_owned();

    let data = KeyPayload {
        creds: Credentials {
            username: username.clone(),
            password,
        },
        key: Key {
            name: key.clone(),
            value: None,
        },
    };

    let headers = data.to_header();

    let response = get_client()?
        .get(url + "/key")
        .header("content-type", "application/json")
        .headers(headers)
        .send()?;

    if response.status() == StatusCode::OK {
        let value = response
            .text()
            .map_err(|_| anyhow!("empty response body"))?;

        println!("{key} = {value}");
    } else if response.status() == StatusCode::NOT_FOUND {
        println!("Key '{}' not found", key);
    } else {
        let status = response.status();
        let text = response.text().unwrap_or_default();
        println!("Error {}: {}", status, text);
    }

    Ok(())
}

pub fn get_all_keys(_matches: &ArgMatches) -> Result<()> {
    let stdin = std::io::stdin();

    let username = try_or_read_username(&stdin)?;
    let password = try_or_read_password(&username, &stdin)?;

    let url = env!("SERVER_URL").trim_end_matches("/").to_owned();

    let data = Credentials {
        username: username.clone(),
        password,
    };

    let headers = data.to_header();

    let response = get_client()?
        .get(url + "/key/list")
        .header("content-type", "application/json")
        .headers(headers)
        .send()?;

    if response.status() == StatusCode::OK {
        let entries = response
            .json::<Vec<String>>()
            .map_err(|_| anyhow!("empty response body"))?;

        for entry in entries {
            println!("{}", entry);
        }
    } else if response.status() == StatusCode::NOT_FOUND {
        println!("No keys stored");
    } else {
        let status = response.status();
        let text = response.text().unwrap_or_default();
        println!("Error {}: {}", status, text);
    }

    Ok(())
}

pub fn set_key(matches: &ArgMatches) -> Result<()> {
    let stdin = std::io::stdin();

    let username = try_or_read_username(&stdin)?;
    let password = try_or_read_password(&username, &stdin)?;

    let key = matches
        .get_one::<String>("name")
        .ok_or_else(|| anyhow!("no key name provided"))?;

    let mut value = String::new();
    print!("Enter value: ");
    stdin
        .read_line(&mut value)
        .map_err(|_| anyhow!("failed to read value"))?;
    let value = value.trim().to_owned();

    let url = env!("SERVER_URL").trim_end_matches("/").to_owned();

    let body = KeyPayload {
        creds: Credentials {
            username: username.clone(),
            password,
        },
        key: Key {
            name: key.clone(),
            value: Some(value.clone()),
        },
    };

    let body = serde_json::to_string(&body).map_err(|_| anyhow!("Failed to serialize payload"))?;

    let response = get_client()?
        .post(url + "/key")
        .header("content-type", "application/json")
        .body(body)
        .send()?;

    if response.status() == StatusCode::CREATED {
        println!("Key saved successfully");
    } else if response.status() == StatusCode::CONFLICT {
        println!("Key '{}' already exists", key);
    } else {
        let status = response.status();
        let text = response
            .text()
            .unwrap_or_else(|_| "unknown error".to_string());
        println!("Error {}: {}", status, text);
    }

    Ok(())
}

pub fn change_key(matches: &ArgMatches) -> Result<()> {
    let stdin = std::io::stdin();

    let username = try_or_read_username(&stdin)?;
    let password = try_or_read_password(&username, &stdin)?;

    let key = matches
        .get_one::<String>("name")
        .ok_or_else(|| anyhow!("no key name provided"))?;

    let mut value = String::new();
    print!("Enter value: ");
    stdin
        .read_line(&mut value)
        .map_err(|_| anyhow!("failed to read value"))?;
    let value = value.trim().to_owned();

    let url = env!("SERVER_URL").trim_end_matches("/").to_owned();

    let body = ChangeKeyPayload {
        creds: Credentials {
            username: username.clone(),
            password,
        },
        name: key.clone(),
        new_value: value.clone(),
    };

    let body = serde_json::to_string(&body).map_err(|_| anyhow!("Failed to serialize payload"))?;

    let response = get_client()?
        .put(url + "/key")
        .header("content-type", "application/json")
        .body(body)
        .send()?;

    if response.status() == StatusCode::CREATED {
        println!("Key changed successfully");
    } else if response.status() == StatusCode::NOT_FOUND {
        println!("Key '{}' not found", key);
    } else {
        let status = response.status();
        let text = response
            .text()
            .unwrap_or_else(|_| "unknown error".to_string());
        println!("Error {}: {}", status, text);
    }
    Ok(())
}

pub fn delete_key(matches: &ArgMatches) -> Result<()> {
    let stdin = std::io::stdin();

    let username = try_or_read_username(&stdin)?;
    let password = try_or_read_password(&username, &stdin)?;

    let key = matches
        .get_one::<String>("name")
        .ok_or_else(|| anyhow!("no key name provided"))?;

    print!("Are you sure you want to delete the key '{}'? (y/N): ", key);
    let mut confirmation = String::new();
    stdin
        .read_line(&mut confirmation)
        .map_err(|_| anyhow!("failed to read confirmation"))?;
    if confirmation.trim() != "y" {
        println!("Aborted");
        return Ok(());
    }

    let url = env!("SERVER_URL").trim_end_matches("/").to_owned();

    let data = KeyPayload {
        creds: Credentials {
            username: username.clone(),
            password,
        },
        key: Key {
            name: key.clone(),
            value: None,
        },
    };

    let headers = data.to_header();

    let response = get_client()?
        .delete(url + "/key")
        .header("content-type", "application/json")
        .headers(headers)
        .send()?;

    if response.status() == StatusCode::OK {
        println!("Key deleted successfully");
    } else {
        let status = response.status();
        let text = response
            .text()
            .unwrap_or_else(|_| "unknown error".to_string());
        println!("Error {}: {}", status, text);
    }

    Ok(())
}
