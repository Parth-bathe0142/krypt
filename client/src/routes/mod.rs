use anyhow::Result;
use clap::ArgMatches;
use reqwest::StatusCode;

pub mod accounts;
pub use accounts::*;

pub mod keys;
pub use keys::*;

use crate::util::{get_client, get_url, handle_unknown_response};

pub(crate) fn ping(_matches: &ArgMatches) -> Result<()> {
    let url = get_url();

    let response = get_client()?.post(url + "/ping").send()?;

    if response.status() == StatusCode::OK {
        let text = response.text().unwrap_or_default();
        println!("{text}");
    } else {
        handle_unknown_response(response);
    }
    Ok(())
}
