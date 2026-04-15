use anyhow::{Result, anyhow};
use clap::Command;

pub fn run(command: Command) -> Result<()> {
    let matches = command.get_matches();
    
    let Some((subcommand, sub_matches)) = matches.subcommand() else {
        return Err(anyhow!("missing sub command"))
    };
    
    match subcommand {
        _ => Err(anyhow!("unknown sub command"))
    }
}