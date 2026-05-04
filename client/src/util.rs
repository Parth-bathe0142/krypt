use std::io::Stdin;

use anyhow::{Result, anyhow};
use reqwest::header::HeaderMap;
use shared::models::{Credentials, KeyPayload};

use crate::{config::get_username, keyring::get_password};

pub(crate) fn try_or_read_username(stdin: &Stdin) -> Result<String> {
    match get_username() {
        Ok(username) => Ok(username),
        Err(err) => {
            println!("Error fetching username from config: {err}");
            println!("\nEnter username manually: ");

            let mut username = String::new();
            stdin.read_line(&mut username)?;

            Ok(username
                .split_whitespace()
                .next()
                .ok_or_else(|| anyhow!("Username cannot be empty"))?
                .to_owned())
        }
    }
}

pub(crate) fn try_or_read_password(username: &str, stdin: &Stdin) -> Result<String> {
    match get_password(&username) {
        Ok(password) => Ok(password),
        Err(err) => {
            println!("could not get password from keyring: {err}");
            println!("\nEnter username manually: ");

            let mut password = String::new();
            stdin.read_line(&mut password)?;

            Ok(password
                .split_whitespace()
                .next()
                .ok_or_else(|| anyhow!("password cannot be empty"))?
                .to_owned())
        }
    }
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
