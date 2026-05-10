use anyhow::{Result, anyhow};
use clap::Command;

use crate::{routes::{
    change_key, change_password, delete_account, delete_key, get_all_keys, get_key, login, ping, set_key, signup
}, tasks::handle_tasks};

mod config;
mod keyring;
mod routes;
mod tasks;
mod util;

pub fn run(command: Command) -> Result<()> {
    let matches = command.get_matches();

    let Some((subcommand, sub_matches)) = matches.subcommand() else {
        return Err(anyhow!("missing sub command"));
    };

    match subcommand {
        "ping" => ping(sub_matches),
        "signup" => signup(sub_matches),
        "login" => login(sub_matches),
        "chpassword" => change_password(sub_matches),
        "delete-account" => delete_account(sub_matches),
        "get" => get_key(sub_matches),
        "list" => get_all_keys(sub_matches),
        "set" => set_key(sub_matches),
        "change" => change_key(sub_matches),
        "delete" => delete_key(sub_matches),

        "config" => handle_tasks(sub_matches),
        cmd @ _ => Err(anyhow!("unknown sub command: {cmd}")),
    }
}
