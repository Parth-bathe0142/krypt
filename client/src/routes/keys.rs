use anyhow::{Result, anyhow};
use clap::ArgMatches;
use reqwest::StatusCode;
use shared::models::{ChangeKeyPayload, Credentials, Key, KeyPayload, ToJson};

use crate::{
    clipboard,
    util::{
        ToHeader, confirm, get_client, get_url, handle_internal_error, handle_unauthorized,
        handle_unknown_response, prompt, try_or_read_password, try_or_read_username,
    },
};

pub fn get_key(matches: &ArgMatches) -> Result<()> {
    let stdin = std::io::stdin();

    let username = try_or_read_username(&stdin)?;
    let password = try_or_read_password(&username)?;

    let key = matches
        .get_one::<String>("name")
        .ok_or_else(|| anyhow!("no key name provided"))?;

    let url = get_url();

    let headers = KeyPayload::new(
        Credentials::new(username, password),
        Key::new(key.clone(), None),
    )
    .to_header();

    let response = get_client()?
        .get(url + "/key")
        .header("content-type", "application/json")
        .headers(headers)
        .send()?;

    match response.status() {
        StatusCode::OK => {
            let value = response
                .text()
                .map_err(|_| anyhow!("empty response body"))?;

            match clipboard::copy(&value) {
                Ok(_) => (),
                Err(err) => {
                    println!("Failed to copy value to clipboard: {err}");
                    println!("Print to console?: ")
                }
            }
        }
        StatusCode::NOT_FOUND => println!("Key '{}' not found", key),
        StatusCode::UNAUTHORIZED => handle_unauthorized(response),
        StatusCode::TOO_MANY_REQUESTS => println!("Too many requests, try again later"),
        StatusCode::BAD_REQUEST => println!("Bad request"),
        StatusCode::INTERNAL_SERVER_ERROR => handle_internal_error(response),
        _ => handle_unknown_response(response),
    }

    Ok(())
}

pub fn get_all_keys(_matches: &ArgMatches) -> Result<()> {
    let stdin = std::io::stdin();

    let username = try_or_read_username(&stdin)?;
    let password = try_or_read_password(&username)?;

    let url = get_url();

    let headers = Credentials::new(username.clone(), password).to_header();

    let response = get_client()?
        .get(url + "/key/list")
        .header("content-type", "application/json")
        .headers(headers)
        .send()?;

    match response.status() {
        StatusCode::OK => {
            let entries = response
                .json::<Vec<String>>()
                .map_err(|_| anyhow!("empty response body"))?;

            for entry in entries {
                println!("{}", entry);
            }
        }
        StatusCode::NOT_FOUND => println!("No keys stored"),
        StatusCode::UNAUTHORIZED => handle_unauthorized(response),
        StatusCode::TOO_MANY_REQUESTS => println!("Too many requests, try again later"),
        StatusCode::BAD_REQUEST => println!("Bad request"),
        StatusCode::INTERNAL_SERVER_ERROR => handle_internal_error(response),
        _ => handle_unknown_response(response),
    }

    Ok(())
}

pub fn set_key(matches: &ArgMatches) -> Result<()> {
    let stdin = std::io::stdin();

    let username = try_or_read_username(&stdin)?;
    let password = try_or_read_password(&username)?;

    let key = matches
        .get_one::<String>("name")
        .ok_or_else(|| anyhow!("no key name provided"))?;

    let value = prompt("Enter value", &stdin)?;

    let url = get_url();
    let body = KeyPayload::new(
        Credentials::new(username, password),
        Key::new(key.clone(), Some(value)),
    )
    .to_json_string()?;

    let response = get_client()?
        .post(url + "/key")
        .header("content-type", "application/json")
        .body(body)
        .send()?;

    match response.status() {
        StatusCode::CREATED => println!("Key saved successfully"),
        StatusCode::CONFLICT => println!("Key '{}' already exists", key),
        StatusCode::UNAUTHORIZED => handle_unauthorized(response),
        StatusCode::TOO_MANY_REQUESTS => println!("Too many requests, try again later"),
        StatusCode::BAD_REQUEST => println!("Bad request"),
        StatusCode::INTERNAL_SERVER_ERROR => handle_internal_error(response),
        _ => handle_unknown_response(response),
    }

    Ok(())
}

pub fn change_key(matches: &ArgMatches) -> Result<()> {
    let stdin = std::io::stdin();

    let username = try_or_read_username(&stdin)?;
    let password = try_or_read_password(&username)?;

    let key = matches
        .get_one::<String>("name")
        .ok_or_else(|| anyhow!("no key name provided"))?;

    let value = prompt("Enter value", &stdin)?;

    let url = get_url();

    let body = ChangeKeyPayload::new(Credentials::new(username, password), key.clone(), value)
        .to_json_string()?;

    let response = get_client()?
        .put(url + "/key")
        .header("content-type", "application/json")
        .body(body)
        .send()?;

    match response.status() {
        StatusCode::CREATED => println!("Key changed successfully"),
        StatusCode::NOT_FOUND => println!("Key '{key}' does not exist"),
        StatusCode::UNAUTHORIZED => handle_unauthorized(response),
        StatusCode::TOO_MANY_REQUESTS => println!("Too many requests, try again later"),
        StatusCode::BAD_REQUEST => println!("Bad request"),
        StatusCode::INTERNAL_SERVER_ERROR => handle_internal_error(response),
        _ => handle_unknown_response(response),
    }
    Ok(())
}

pub fn delete_key(matches: &ArgMatches) -> Result<()> {
    let stdin = std::io::stdin();

    let username = try_or_read_username(&stdin)?;
    let password = try_or_read_password(&username)?;

    let key = matches
        .get_one::<String>("name")
        .ok_or_else(|| anyhow!("no key name provided"))?;

    if confirm(
        &format!("Are you sure you want to delete the key '{key}'?"),
        true,
        &stdin,
    )? {
        println!("Aborted");
        return Ok(());
    }

    let url = get_url();

    let headers = KeyPayload::new(
        Credentials::new(username, password),
        Key::new(key.clone(), None),
    )
    .to_header();

    let response = get_client()?
        .delete(url + "/key")
        .header("content-type", "application/json")
        .headers(headers)
        .send()?;

    match response.status() {
        StatusCode::OK => println!("Key deleted successfully"),
        StatusCode::NOT_FOUND => println!("Key '{key}' does not exist"),
        StatusCode::UNAUTHORIZED => handle_unauthorized(response),
        StatusCode::TOO_MANY_REQUESTS => println!("Too many requests, try again later"),
        StatusCode::BAD_REQUEST => println!("Bad request"),
        StatusCode::INTERNAL_SERVER_ERROR => handle_internal_error(response),
        _ => handle_unknown_response(response),
    }
    Ok(())
}
