use anyhow::Result;
use clap::{Arg, ArgAction, Command};
use krypt::run;

fn main() -> Result<()> {
    let command = Command::new("krypt")
        .about("A simple online keyring for storing credentials accessible on multiple devices through the cli")
        .author("Parth-Bathe0142 <parth.bathe0142@gmail.com>")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(Command::new("ping"))
        .subcommand(Command::new("signup"))
        .subcommand(Command::new("login"))
        .subcommand(Command::new("logout"))
        .subcommand(Command::new("chpassword"))
        .subcommand(Command::new("delete-account"))
        .subcommand(
            Command::new("get")
                .arg(
                    Arg::new("name")
                        .index(1)
                        .value_name("KEY")
                        .required(true)
                        .help("The name of the key")
                )
        ).subcommand(Command::new("list"))
        .subcommand(
            Command::new("set")
                .arg(
                    Arg::new("name")
                        .index(1)
                        .value_name("KEY")
                        .required(true)
                        .help("The name of the new key")
                )
        ).subcommand(
            Command::new("change")
                .arg(
                    Arg::new("name")
                        .index(1)
                        .value_name("KEY")
                        .required(true)
                        .help("The name of the key to change")
                )
        ).subcommand(
            Command::new("delete")
                .arg(
                    Arg::new("name")
                        .index(1)
                        .value_name("KEY")
                        .required(true)
                        .help("The name of the key to delete")
                )
        ).subcommand(
            Command::new("config")
                .subcommand(Command::new("set-url")
                    .arg(
                        Arg::new("url")
                            .index(1)
                            .value_name("URL")
                            .conflicts_with("default")
                            .help("The new URL of the deployed server")
                    ).arg(
                        Arg::new("default")
                            .short('d')
                            .long("default")
                            .conflicts_with("url")
                            .action(ArgAction::SetTrue)
                            .help("Set URL back to the default URL from build.rs")
                    )
                ).subcommand(
                    Command::new("set-copy-timeout")
                        .arg(
                            Arg::new("timeout")
                                .index(1)
                                .value_name("TIMEOUT")
                                .conflicts_with("none")
                                .value_parser(clap::value_parser!(u64).range(1..60))
                                .help("The timeout in seconds for keeping the key in the clipboard before removal")
                        ).arg(
                            Arg::new("none")
                                .short('n')
                                .long("none")
                                .conflicts_with("timeout")
                                .action(ArgAction::SetTrue)
                                .help("Disables the timeout and keeps the key in the clipboard indefinitely")
                        )
                )
        );

    run(command)
}
