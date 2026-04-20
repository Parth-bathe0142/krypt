use anyhow::Result;
use clap::{Arg, ArgAction, Command};
use krypt::run;

fn main() -> Result<()> {
    let command = Command::new("krypt")
        .about("An simple online keyring for storing credentials accessible on multiple devices throught the cli")
        .author("Parth-Bathe0142 <parth.bathe0142@gmail.com>")
        .version("1.0")
        .subcommand(
            Command::new("signup")
                .arg(
                    Arg::new("username")
                        .short('u')
                        .long("username")
                        .required(true)
                        .help("new krypt username, must be 3..32 characters")
                ).arg(
                    Arg::new("password")
                        .short('p')
                        .long("password")
                        .required(true)
                        .help("new krypt password, must be 8..32 characters")
                )
        ).subcommand(
            Command::new("login")
                .arg(
                    Arg::new("username")
                        .short('u')
                        .long("username")
                        .required(true)
                )
                .arg(
                    Arg::new("password")
                        .short('p')
                        .long("password")
                        .required(true)
                )
        ).subcommand(Command::new("logout"))
        .subcommand(
            Command::new("chpassword")
                .arg(
                    Arg::new("old")
                        .required(true)
                ).arg(
                    Arg::new("new")
                        .required(true)
                )
        ).subcommand(
            Command::new("get")
                .arg(
                    Arg::new("name")
                        .index(1)
                        .conflicts_with("all")
                        .help("The name of the key")
                ).arg(
                    Arg::new("all")
                        .long("all")
                        .required(false)
                        .conflicts_with("name")
                        .action(ArgAction::SetTrue)

                )
        ).subcommand(
            Command::new("set")
                .arg(
                    Arg::new("name")
                        .index(1)
                        .required(true)
                        .help("The name of the new key")
                ).arg(
                    Arg::new("value")
                        .index(2)
                        .required(true)
                        .help("The new value of the key")
                )
        ).subcommand(
            Command::new("change")
                .arg(
                    Arg::new("name")
                        .index(1)
                        .required(true)
                        .help("The name of the key to change")
                ).arg(
                    Arg::new("new-value")
                        .long("to")
                        .required(true)
                        .help("The new value for the given key")
                )
        ).subcommand(
            Command::new("delete")
                .arg(
                    Arg::new("name")
                        .index(1)
                        .required(true)
                        .help("The name of the key to delete")
                )
                .subcommand(Command::new("account"))
                .args_conflicts_with_subcommands(true)
        );

    run(command)
}
