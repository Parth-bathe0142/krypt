use anyhow::{Result, anyhow};
use clap::ArgMatches;
use reqwest::StatusCode;

use crate::{config::add_entry, util::get_client};

pub(crate) fn handle_tasks(matches: &ArgMatches) -> Result<()> {
    let Some((subcommand, sub_matches)) = matches.subcommand() else {
        return Err(anyhow!("No subcommand provided"));
    };

    match subcommand {
        "set-url" => {
            let url = sub_matches.get_one::<String>("url");

            if let Some(url) = url {
                change_url(url)
            } else if sub_matches.get_flag("default") {
                set_default_url()
            } else {
                Err(anyhow!("No arguments provided"))
            }
        }
        "set-copy-timeout" => {
            let timeout = sub_matches.get_one("timeout");

            if let Some(timeout) = timeout {
                set_copy_timeout(*timeout)
            } else if sub_matches.get_flag("none") {
                set_copy_timeout(0)
            } else {
                Err(anyhow!("No arguments provided"))
            }
        }
        _ => Err(anyhow!("Unknown sub command")),
    }
}

pub(crate) fn change_url(url: &str) -> Result<()> {
    let url = url.trim_end_matches('/');
    let client = get_client()?;

    let response = match client.get(url.to_owned() + "/ping").send() {
        Ok(response) => response,
        Err(err) => {
            eprintln!(
                "Could not connect to the server on the given url, stored url was not changed: {err}"
            );
            return Ok(());
        }
    };

    let status = response.status();
    let text = response.text().unwrap_or_default();

    if status == StatusCode::NOT_FOUND {
        eprintln!("Server is reachable, but ping failed. Is the url correct?");
        return Ok(());
    } else if status != StatusCode::OK {
        eprintln!("Unexpected response: {}", status);
        return Ok(());
    }

    if text == "pong" {
        println!("Server is up and running");
    } else {
        eprintln!("Server is reachable, but ping failed. Is the url correct?");
        return Ok(());
    }

    let old = add_entry("server", "url", url)?;
    if let Some(old) = old {
        println!("Changed from {old} to {url}");
    } else {
        println!("Set url to {url}");
    }

    Ok(())
}

pub(crate) fn set_default_url() -> Result<()> {
    let default = env!("SERVER_URL");
    let old = add_entry("server", "url", default)?;
    if let Some(old) = old {
        println!("Changed from {old} to {default}");
    } else {
        println!("Set url to {default}");
    }

    Ok(())
}

/// a timeout of 0 disables the clipboard clearing
pub(crate) fn set_copy_timeout(timeout: u64) -> Result<()> {
    let old = if timeout == 0 {
        add_entry("copy", "timeout", "None")?
    } else {
        add_entry("copy", "timeout", &timeout.to_string())?
    };
    if let Some(old) = old {
        println!("Changed copy timeout from {old} to {timeout}");
    } else {
        println!("Set copy timeout to {timeout}");
    }

    Ok(())
}