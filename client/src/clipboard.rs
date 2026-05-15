use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Duration;

use anyhow::{Result, anyhow};
use arboard::Clipboard;

use crate::config;

const DEFAULT_TIMEOUT: u64 = 30;
const UTILITIES: &[&dyn Utility] = &[&WlCopy, &Arboard];

pub(crate) fn copy(text: &str) -> Result<()> {
    let timeout = match config::get_value("copy", "timeout") {
        Ok(Some(v)) if v == "None" => 0,
        Ok(Some(v)) => v.parse::<u64>().unwrap_or(DEFAULT_TIMEOUT),
        Err(_) | Ok(None) => DEFAULT_TIMEOUT,
    };

    let utility = get_working_utility()?;
    utility.copy(text)?;
    println!("Value has been copied to clipboard.");

    if timeout != 0 {
        println!("Clipboard will be cleared in {timeout} seconds.");
        std::thread::sleep(Duration::from_secs(timeout));

        let value = utility.read()?;
        if value.trim() == text {
            if utility.clear().is_ok() {
                println!("Clipboard has been cleared");
            } else {
                println!("Failed to clear clipboard");
            }
        } else {
            println!("Clipboard value has changed, not clearing.");
        }
    }

    Ok(())
}

fn get_working_utility() -> Result<&'static dyn Utility> {
    if let Ok(Some(name)) = config::get_value("copy", "utility") {
        let utility = UTILITIES.iter().find(|u| u.name() == name).map(|u| *u);

        if let Some(utility) = utility {
            return Ok(utility);
        }
    }

    for utility in UTILITIES {
        if utility.check() {
            let name = utility.name();
            let _ = config::add_entry("copy", "utility", name);

            return Ok(*utility);
        }
    }

    Err(anyhow!("No clipboard utility works"))
}

trait Utility {
    fn name(&self) -> &'static str;
    fn copy(&self, text: &str) -> Result<()>;
    fn read(&self) -> Result<String>;
    fn clear(&self) -> Result<()>;
    fn check(&self) -> bool;
}

struct WlCopy;
impl Utility for WlCopy {
    fn name(&self) -> &'static str {
        "wl-copy"
    }

    fn copy(&self, text: &str) -> Result<()> {
        let mut child = Command::new("wl-copy")
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| anyhow!("wl-copy not available: {e}"))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes())?;
        }

        Ok(())
    }

    fn read(&self) -> Result<String> {
        let output = Command::new("wl-paste")
            .output()
            .map_err(|e| anyhow!("wl-paste not available: {e}"))?;

        if !output.status.success() {
            return Err(anyhow!("wl-paste failed"));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn clear(&self) -> Result<()> {
        let status = Command::new("wl-copy")
            .arg("--clear")
            .status()
            .map_err(|_| anyhow!("failed to clear wl clipboard:"))?;

        if !status.success() {
            return Err(anyhow!("wl-copy --clear failed"));
        }

        Ok(())
    }

    fn check(&self) -> bool {
        Command::new("wl-copy")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

struct Arboard;
impl Utility for Arboard {
    fn name(&self) -> &'static str {
        "arboard"
    }

    fn copy(&self, text: &str) -> Result<()> {
        let mut clipboard = Clipboard::new()?;
        clipboard.set_text(text)?;
        Ok(())
    }

    fn read(&self) -> Result<String> {
        let mut clipboard = arboard::Clipboard::new()?;
        clipboard.get_text().map_err(|e| anyhow!("{e}"))
    }

    fn clear(&self) -> Result<()> {
        self.copy("")
    }

    fn check(&self) -> bool {
        Clipboard::new().is_ok()
    }
}
