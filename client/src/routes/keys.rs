use anyhow::{Result, anyhow};
use clap::ArgMatches;
use reqwest::StatusCode;
use shared::models::{ChangeKeyPayload, Credentials, Key, KeyPayload, ToJson};

use crate::util::{
    ToHeader, get_client, get_url, handle_unknown_response, prompt, try_or_read_password,
    try_or_read_username,
};

pub fn get_key(matches: &ArgMatches) -> Result<()> {
    let stdin = std::io::stdin();

    let username = try_or_read_username(&stdin)?;
    let password = try_or_read_password(&username, &stdin)?;

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

    if response.status() == StatusCode::OK {
        let value = response
            .text()
            .map_err(|_| anyhow!("empty response body"))?;

        println!("{key} = {value}");
    } else if response.status() == StatusCode::NOT_FOUND {
        println!("Key '{}' not found", key);
    } else {
        handle_unknown_response(response);
    }

    Ok(())
}

pub fn get_all_keys(_matches: &ArgMatches) -> Result<()> {
    let stdin = std::io::stdin();

    let username = try_or_read_username(&stdin)?;
    let password = try_or_read_password(&username, &stdin)?;

    let url = get_url();

    let headers = Credentials::new(username.clone(), password).to_header();

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
        handle_unknown_response(response);
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

    let value = prompt("Enter value: ", &stdin)?;

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

    if response.status() == StatusCode::CREATED {
        println!("Key saved successfully");
    } else if response.status() == StatusCode::CONFLICT {
        println!("Key '{}' already exists", key);
    } else {
        handle_unknown_response(response);
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

    let value = prompt("Enter value: ", &stdin)?;

    let url = get_url();

    let body = ChangeKeyPayload::new(Credentials::new(username, password), key.clone(), value)
        .to_json_string()?;

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
        handle_unknown_response(response);
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

    if response.status() == StatusCode::OK {
        println!("Key deleted successfully");
    } else {
        handle_unknown_response(response);
    }
    Ok(())
}
