use std::{io::{Stdin, Write, stdout}, time::Duration};

use anyhow::{Result, anyhow};
use reqwest::{blocking::Response, header::HeaderMap};
use shared::models::{Credentials, KeyPayload};

use crate::{config::get_username, keyring::get_password};

pub(crate) fn prompt(message: &str, stdin: &Stdin) -> Result<String> {
    let mut tries = 0;
    loop {
        let mut str = String::new();
        print!("{}: ", message);

        stdout().flush()?;
        stdin
            .read_line(&mut str)
            .map_err(|_| anyhow!("failed to read username"))?;
        let str = str.trim();

        if !str.is_empty() {
            break Ok(str.to_owned());
        }

        println!("Username cannot be empty");
        tries += 1;
        if tries >= 3 {
            break Err(anyhow!("Too many attempts"));
        }
    }
}

pub(crate) fn try_or_read_username(stdin: &Stdin) -> Result<String> {
    match get_username() {
        Ok(username) => Ok(username),
        Err(err) => {
            println!("could not get username from config: {err}");
            prompt("Enter username", stdin)
        }
    }
}

pub(crate) fn try_or_read_password(username: &str, stdin: &Stdin) -> Result<String> {
    match get_password(&username) {
        Ok(password) => Ok(password),
        Err(err) => {
            println!("could not get password from keyring: {err}");
            prompt("Enter password", stdin)
        }
    }
}

pub(crate) fn get_url() -> String {
    env!("SERVER_URL").trim_end_matches("/").to_owned()
}

pub(crate) fn get_client() -> Result<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| anyhow::anyhow!(e))
}

pub(crate) trait ToHeader {
    fn to_header(&self) -> HeaderMap;
}

impl ToHeader for Credentials {
    fn to_header(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("username", self.username.parse().unwrap());
        headers.insert("password", self.password.parse().unwrap());
        headers
    }
}

impl ToHeader for KeyPayload {
    fn to_header(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("username", self.creds.username.parse().unwrap());
        headers.insert("password", self.creds.password.parse().unwrap());
        headers.insert("key", self.key.name.parse().unwrap());
        headers
    }
}

pub(crate) fn handle_unknown_response(response: Response) {
    let status = response.status();
    let text = response.text().unwrap_or_default();
    println!("Error {}: {}", status, text);
}
