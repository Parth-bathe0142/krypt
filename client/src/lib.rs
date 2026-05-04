use anyhow::{Result, anyhow};
use clap::Command;

use crate::{
    routes::{change_password, delete_account, login, signup, get_key, get_all_keys, set_key, change_key, delete_key},
};

mod routes;
mod config;
mod keyring;
mod util;

pub fn run(command: Command) -> Result<()> {
    let matches = command.get_matches();

    let Some((subcommand, sub_matches)) = matches.subcommand() else {
        return Err(anyhow!("missing sub command"));
    };

    match subcommand {
        "signup" => signup(sub_matches),
        "login" => login(sub_matches),
        "chpassword" => change_password(sub_matches),
        "delete-account" => delete_account(sub_matches),
        "get" => get_key(sub_matches),
        "list" => get_all_keys(sub_matches),
        "set" => set_key(sub_matches),
        "change" => change_key(sub_matches),
        "delete" => delete_key(sub_matches),

        _ => Err(anyhow!("unknown sub command")),
    }
}

