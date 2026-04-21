use std::io::Stdin;

use anyhow::{Result, anyhow};

use crate::{config::get_username, keyring::get_password};

pub(crate) fn try_or_read_username(stdin: &Stdin) -> Result<String> {
    match get_username() {
        Ok(username) => Ok(username),
        Err(err) => {
            println!("Error fetching username from config: {err}");
            print!("\nEnter username manually: ");

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
            print!("\nEnter username manually: ");

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