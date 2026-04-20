use std::time::Duration;

use anyhow::{Result, anyhow};
use clap::{ArgMatches, Command};
use shared::{
    models::{ChangePasswordPayload, Credentials},
    validate_password, validate_username,
};

use crate::{
    config::{clear_username, get_username, set_username},
    keyring::{clear_password, get_password, save},
};

mod config;
mod keyring;

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

    let response = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?
        .post(url + "/account")
        .header("content-type", "application/json")
        .body(body)
        .send()?;

    if response.status() == 201 {
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
    } else {
        let status = response.status();
        let text = response
            .text()
            .unwrap_or_else(|_| "unknown error".to_string());
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

    let response = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?
        .get(url + "/account")
        .header("content-type", "application/json")
        .body(body)
        .send()?;

    if response.status() == 302 {
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
    } else {
        let status = response.status();
        let text = response
            .text()
            .unwrap_or_else(|_| "unknown error".to_string());
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

    let username = match get_username() {
        Ok(username) => username,
        Err(err) => {
            println!("Error fetching username from config: {err}");
            print!("\nEnter username manually: ");

            let mut username = String::new();
            stdin.read_line(&mut username)?;

            username
                .split_whitespace()
                .next()
                .ok_or_else(|| anyhow!("Username cannot be empty"))?
                .to_owned()
        }
    };

    let url = env!("SERVER_URL").trim_end_matches("/").to_owned();

    let body = ChangePasswordPayload {
        creds: Credentials {
            username: username.clone(),
            password: old,
        },
        new_password: new.clone(),
    };

    let body = serde_json::to_string(&body).map_err(|_| anyhow!("Failed to serialize payload"))?;

    let response = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?
        .put(url + "/account")
        .header("content-type", "application/json")
        .body(body)
        .send()?;

    if response.status() == 200 {
        println!("Password changed successfully");
        save(&username, &new)?;
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

    let (username, found) = match get_username() {
        Ok(username) => (username, true),
        Err(err) => {
            println!("Error fetching username from config: {err}");
            print!("\nEnter username manually: ");

            let mut username = String::new();
            stdin.read_line(&mut username)?;

            let username = username
                .split_whitespace()
                .next()
                .ok_or_else(|| anyhow!("Username cannot be empty"))?
                .to_owned();

            (username, false)
        }
    };

    let password = if found {
        get_password(&username)?
    } else {
        let mut password = String::new();
        stdin.read_line(&mut password)?;

        print!("Enter password: ");
        password
            .split_whitespace()
            .next()
            .ok_or_else(|| anyhow!("Username cannot be empty"))?
            .to_owned()
    };

    let url = env!("SERVER_URL").trim_end_matches("/").to_owned();

    let body = Credentials {
        username: username.clone(),
        password,
    };

    let body = serde_json::to_string(&body).map_err(|_| anyhow!("Failed to serialize payload"))?;

    let response = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?
        .delete(url + "/account")
        .header("content-type", "application/json")
        .body(body)
        .send()?;

    if response.status() == 200 {
        println!("Account deleted successfully");

        clear_password(&username)?;
        clear_username()?;
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
    Ok(())
}

fn get_all_keys(matches: &ArgMatches) -> Result<()> {
    Ok(())
}

fn set_key(matches: &ArgMatches) -> Result<()> {
    Ok(())
}

fn change_key(matches: &ArgMatches) -> Result<()> {
    Ok(())
}

fn delete_key(matches: &ArgMatches) -> Result<()> {
    Ok(())
}
