use anyhow::{Result, anyhow};

pub(crate) fn save(username: &str, password: &str) -> Result<()> {
    let entry = keyring::Entry::new("krypt", username)
        .map_err(|err| anyhow!("failed to connect with keyring: {err}"))?;
    
    entry
        .set_password(password)
        .map_err(|err| anyhow!("Failed rto save password to keyring: {err}"))?;
    
    Ok(())
}

pub(crate) fn get_password(username: &str) -> Result<String> {
    let entry = keyring::Entry::new("krypt", username)
        .map_err(|err| anyhow!("failed to connect with keyring: {err}"))?;
    
    entry
        .get_password()
        .map_err(|err| anyhow!("Failed rto save password to keyring: {err}"))
}

pub(crate) fn clear_password(username: &str) -> Result<()> {
    let entry = keyring::Entry::new("krypt", username)
        .map_err(|err| anyhow!("failed to connect with keyring: {err}"))?;
    
    entry.delete_credential()
        .map_err(|err| anyhow!("Failed to remove credentials from keyring: {err}"))
}

