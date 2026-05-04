use anyhow::{Result, anyhow};
use clap::{ArgMatches, Command};
use reqwest::StatusCode;
use shared::{
    models::{ChangeKeyPayload, ChangePasswordPayload, Credentials, Key, KeyPayload},
    validate_password, validate_username,
};

use crate::{
    config::{clear_username, set_username},
    keyring::{clear_password, save},
    util::{ToHeader, get_client, try_or_read_password, try_or_read_username},
};

mod config;
mod keyring;
mod util;

pub fn run(command: Command) -> Result<()> {
    let matches = command.get_matches();

    let Some((subcommand, sub_matches)) = matches.subcommand() else {
        return Err(anyhow!("missing sub command"));
    };

    match subcommand {
        "signup" => signup(sub_matches),
        "login" => login(sub_matches),
        "chpassword" => change_password(sub_matches),
        "get" => {
            if sub_matches.get_flag("all") {
                get_all_keys(sub_matches)
            } else {
                get_key(sub_matches)
            }
        }
        "set" => set_key(sub_matches),
        "change" => change_key(sub_matches),
        "delete" => {
            if let Some(("account", _)) = sub_matches.subcommand() {
                delete_account(sub_matches)
            } else {
                delete_key(sub_matches)
            }
        }
        _ => Err(anyhow!("unknown sub command")),
    }
}

fn signup(matches: &ArgMatches) -> Result<()> {
    let username = matches
        .get_one::<String>("username")
        .ok_or_else(|| anyhow!("username required"))?
        .to_owned();

    let password = matches
        .get_one::<String>("password")
        .ok_or_else(|| anyhow!("password required"))?
        .to_owned();

    validate_username(&username)?;
    validate_password(&password)?;

    let url = env!("SERVER_URL").trim_end_matches("/").to_owned();
    let body = Credentials {
        username: username.clone(),
        password: password.clone(),
    };

    let body = serde_json::to_string(&body).map_err(|_| anyhow!("failed to serialize body"))?;

    let response = get_client()?
        .post(url + "/account")
        .header("content-type", "application/json")
        .body(body)
        .send()?;

    if response.status() == StatusCode::CREATED {
        println!("Account created successfully");
        println!("Saving credentials to keyring...");

        let saved = save(&username, &password);
        if saved.is_ok() {
            set_username(&username)?;

            println!("Credentials saved successfully")
        } else {
            println!(
                "Failed to save credentials to the device's keyring, you may have to provide credentials with each request"
            );
            return saved;
        }
    } else if response.status() == StatusCode::CONFLICT {
        println!("Username already exists");
    } else {
        let status = response.status();
        let text = response.text().unwrap_or_default();
        println!("Error {}: {}", status, text);
    }

    Ok(())
}

fn login(matches: &ArgMatches) -> Result<()> {
    let username = matches
        .get_one::<String>("username")
        .ok_or_else(|| anyhow!("username required"))?
        .to_owned();

    let password = matches
        .get_one::<String>("password")
        .ok_or_else(|| anyhow!("password required"))?
        .to_owned();

    let url = env!("SERVER_URL").trim_end_matches("/").to_owned();
    let body = Credentials {
        username: username.clone(),
        password: password.clone(),
    };

    let body = serde_json::to_string(&body).map_err(|_| anyhow!("failed to serialize body"))?;

    let response = get_client()?
        .post(url + "/account/login")
        .header("content-type", "application/json")
        .body(body)
        .send()?;

    if response.status() == StatusCode::ACCEPTED {
        println!("Account verified successfully");
        println!("Saving credentials to keyring...");

        set_username(&username)?;

        let saved = save(&username, &password);
        if saved.is_ok() {
            println!("Credentials saved successfully")
        } else {
            println!(
                "Failed to save credentials to the device's keyring, you may have to provide credentials with each request"
            );
            return saved;
        }
    } else if response.status() == StatusCode::UNAUTHORIZED {
        println!("{}", response.text().unwrap_or_default());
    } else {
        let status = response.status();
        let text = response.text().unwrap_or_default();
        println!("Error {}: {}", status, text);
    }

    Ok(())
}

fn change_password(matches: &ArgMatches) -> Result<()> {
    let stdin = std::io::stdin();

    let old = matches
        .get_one::<String>("old")
        .ok_or_else(|| anyhow!("Old password required"))?
        .to_owned();

    let new = matches
        .get_one::<String>("new")
        .ok_or_else(|| anyhow!("New password required"))?
        .to_owned();

    let username = try_or_read_username(&stdin)?;

    let url = env!("SERVER_URL").trim_end_matches("/").to_owned();

    let body = ChangePasswordPayload {
        creds: Credentials {
            username: username.clone(),
            password: old,
        },
        new_password: new.clone(),
    };

    let body = serde_json::to_string(&body).map_err(|_| anyhow!("Failed to serialize payload"))?;

    let response = get_client()?
        .put(url + "/account")
        .header("content-type", "application/json")
        .body(body)
        .send()?;

    if response.status() == StatusCode::OK {
        println!("Password changed successfully");
        save(&username, &new)?;
    } else if response.status() == StatusCode::UNAUTHORIZED {
        println!("{}", response.text().unwrap_or_default());
    } else {
        let status = response.status();
        let text = response
            .text()
            .unwrap_or_else(|_| "unknown error".to_string());
        println!("Error {}: {}", status, text);
    }

    Ok(())
}

fn delete_account(_matches: &ArgMatches) -> Result<()> {
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
        .delete(url + "/account")
        .header("content-type", "application/json")
        .headers(headers)
        .send()?;

    if response.status() == StatusCode::OK {
        println!("Account deleted successfully");

        clear_password(&username)?;
        clear_username()?;
    } else if response.status() == StatusCode::UNAUTHORIZED {
        println!("{}", response.text().unwrap_or_default());
    } else {
        let status = response.status();
        let text = response
            .text()
            .unwrap_or_else(|_| "unknown error".to_string());
        println!("Error {}: {}", status, text);
    }

    Ok(())
}

fn get_key(matches: &ArgMatches) -> Result<()> {
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

fn get_all_keys(_matches: &ArgMatches) -> Result<()> {
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

fn set_key(matches: &ArgMatches) -> Result<()> {
    let stdin = std::io::stdin();

    let username = try_or_read_username(&stdin)?;
    let password = try_or_read_password(&username, &stdin)?;

    let key = matches
        .get_one::<String>("name")
        .ok_or_else(|| anyhow!("no key name provided"))?;

    let value = matches
        .get_one::<String>("value")
        .ok_or_else(|| anyhow!("no value provided"))?;

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

fn change_key(matches: &ArgMatches) -> Result<()> {
    let stdin = std::io::stdin();

    let username = try_or_read_username(&stdin)?;
    let password = try_or_read_password(&username, &stdin)?;

    let key = matches
        .get_one::<String>("name")
        .ok_or_else(|| anyhow!("no key name provided"))?;

    let value = matches
        .get_one::<String>("new-value")
        .ok_or_else(|| anyhow!("no value provided"))?;

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

fn delete_key(matches: &ArgMatches) -> Result<()> {
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
