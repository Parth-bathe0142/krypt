use std::io::{Write, stdout};

use anyhow::{Result, anyhow};
use clap::ArgMatches;
use reqwest::StatusCode;
use shared::{
    models::{ChangePasswordPayload, Credentials, ToJson},
    validate_password, validate_username,
};

use crate::{
    config,
    keyring::{clear_password, save},
    util::{
        ToHeader, get_client, get_url, handle_internal_error, handle_unauthorized,
        handle_unknown_response, prompt, try_or_read_password, try_or_read_username,
    },
};

pub fn signup(_matches: &ArgMatches) -> Result<()> {
    let stdin = std::io::stdin();

    let username = prompt("Enter username", &stdin)?;

    let password = rpassword::prompt_password("Enter Password: ")?;
    let confirm = rpassword::prompt_password("Confirm Password: ")?;
    if password != confirm {
        return Err(anyhow!("passwords do not match"));
    }

    validate_username(&username)?;
    validate_password(&password)?;

    let url = get_url();
    let body = Credentials::new(username.clone(), password.clone()).to_json_string()?;

    let response = get_client()?
        .post(url + "/account")
        .header("content-type", "application/json")
        .body(body)
        .send()?;

    match response.status() {
        StatusCode::CREATED => {
            println!("Account created successfully");
            println!("Saving credentials to keyring...");

            let saved = save(&username, &password);
            if saved.is_ok() {
                config::add_entry("", "username", &username)?;

                println!("Credentials saved successfully")
            } else {
                println!(
                    "Failed to save credentials to the device's keyring, you may have to provide credentials with each request"
                );
                return saved;
            }
        }

        StatusCode::CONFLICT => println!("Username already exists"),
        StatusCode::BAD_REQUEST => println!("Bad request"),
        StatusCode::NOT_ACCEPTABLE => println!("Invalid credentials"),
        StatusCode::INTERNAL_SERVER_ERROR => handle_internal_error(response),
        _ => handle_unknown_response(response),
    }

    Ok(())
}

pub fn login(_matches: &ArgMatches) -> Result<()> {
    let stdin = std::io::stdin();

    let username = prompt("Enter username", &stdin)?;
    let password = rpassword::prompt_password("Enter Password: ")?;

    let url = get_url();
    let body = Credentials::new(username.clone(), password.clone()).to_json_string()?;

    let response = get_client()?
        .post(url + "/account/login")
        .header("content-type", "application/json")
        .body(body)
        .send()?;

    match response.status() {
        StatusCode::ACCEPTED => {
            println!("Account verified successfully");
            println!("Saving credentials to keyring...");

            config::add_entry("", "username", &username)?;

            let saved = save(&username, &password);
            if saved.is_ok() {
                println!("Credentials saved successfully")
            } else {
                println!(
                    "Failed to save credentials to the device's keyring, you may have to provide credentials with each request"
                );
                return saved;
            }
        }
        StatusCode::UNAUTHORIZED => handle_unauthorized(response),
        StatusCode::BAD_REQUEST => println!("Bad request"),
        StatusCode::TOO_MANY_REQUESTS => println!("Too many attempts, try again later"),
        StatusCode::INTERNAL_SERVER_ERROR => handle_internal_error(response),
        _ => handle_unknown_response(response),
    }

    Ok(())
}

pub fn change_password(_matches: &ArgMatches) -> Result<()> {
    let stdin = std::io::stdin();

    let username = try_or_read_username(&stdin)?;

    let old = rpassword::prompt_password("Enter old password: ")?;
    let new = rpassword::prompt_password("Enter new password: ")?;

    let url = get_url();
    let body = ChangePasswordPayload::new(Credentials::new(username.clone(), old), new.clone())
        .to_json_string()?;

    let response = get_client()?
        .put(url + "/account")
        .header("content-type", "application/json")
        .body(body)
        .send()?;

    match response.status() {
        StatusCode::OK => {
            println!("Password changed successfully");
            save(&username, &new)?;
        }
        StatusCode::NOT_ACCEPTABLE => {
            println!("{}", response.text().unwrap_or_default());
        }
        StatusCode::UNAUTHORIZED => handle_unauthorized(response),
        StatusCode::BAD_REQUEST => println!("Bad request"),
        StatusCode::TOO_MANY_REQUESTS => println!("Too many attempts, try again later"),
        StatusCode::INTERNAL_SERVER_ERROR => handle_internal_error(response),
        _ => handle_unknown_response(response),
    }

    Ok(())
}

pub fn delete_account(_matches: &ArgMatches) -> Result<()> {
    let stdin = std::io::stdin();

    let username = try_or_read_username(&stdin)?;
    let password = try_or_read_password(&username)?;

    print!("Are you sure you want to delete your account? (y/N): ");
    stdout().flush()?;
    let mut confirmation = String::new();
    stdin
        .read_line(&mut confirmation)
        .map_err(|_| anyhow!("failed to read confirmation"))?;
    if confirmation.trim() != "y" {
        println!("Aborted");
        return Ok(());
    }

    let url = get_url();

    let headers = Credentials::new(username.clone(), password).to_header();

    let response = get_client()?
        .delete(url + "/account")
        .header("content-type", "application/json")
        .headers(headers)
        .send()?;

    match response.status() {
        StatusCode::OK => {
            println!("Account deleted successfully");

            clear_password(&username)?;
            config::clear_value("", "username")?;
        }
        StatusCode::UNAUTHORIZED => handle_unauthorized(response),
        StatusCode::BAD_REQUEST => println!("Bad request"),
        StatusCode::TOO_MANY_REQUESTS => println!("Too many attempts, try again later"),
        StatusCode::INTERNAL_SERVER_ERROR => handle_internal_error(response),
        _ => handle_unknown_response(response),
    }

    Ok(())
}
