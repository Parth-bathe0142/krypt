use anyhow::{Result, anyhow};
use clap::ArgMatches;
use reqwest::StatusCode;
use shared::{models::{ChangePasswordPayload, Credentials}, validate_password, validate_username};

use crate::{config::{clear_username, set_username}, keyring::{clear_password, save}, util::{ToHeader, get_client, try_or_read_password, try_or_read_username}};

pub fn signup(_matches: &ArgMatches) -> Result<()> {
    let stdin = std::io::stdin();

    let mut username = String::new();
    print!("Enter username: ");
    stdin
        .read_line(&mut username)
        .map_err(|_| anyhow!("failed to read username"))?;
    let username = username.trim().to_owned();

    let password = rpassword::prompt_password("Enter Password: ")?;
    let confirm = rpassword::prompt_password("Confirm Password: ")?;
    if password != confirm {
        return Err(anyhow!("passwords do not match"));
    }

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

pub fn login(_matches: &ArgMatches) -> Result<()> {
    let stdin = std::io::stdin();

    let mut username = String::new();
    print!("Enter username: ");
    stdin
        .read_line(&mut username)
        .map_err(|_| anyhow!("failed to read username"))?;
    let username = username.trim().to_owned();

    let password = rpassword::prompt_password("Enter Password: ")?;

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

pub fn change_password(_matches: &ArgMatches) -> Result<()> {
    let stdin = std::io::stdin();

    let username = try_or_read_username(&stdin)?;

    let old = rpassword::prompt_password("Enter old password: ")?;
    let new = rpassword::prompt_password("Enter new password: ")?;

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

pub fn delete_account(_matches: &ArgMatches) -> Result<()> {
    let stdin = std::io::stdin();

    let username = try_or_read_username(&stdin)?;
    let password = try_or_read_password(&username, &stdin)?;

    print!("Are you sure you want to delete your account? (y/N): ");
    let mut confirmation = String::new();
    stdin
        .read_line(&mut confirmation)
        .map_err(|_| anyhow!("failed to read confirmation"))?;
    if confirmation.trim() != "y" {
        println!("Aborted");
        return Ok(());
    }

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
