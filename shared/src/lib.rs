use anyhow::{Result, anyhow};

pub mod models;

pub fn validate_username(username: &str) -> Result<()> {
    if username.is_empty() {
        return Err(anyhow!("Username cannot be empty"));
    }

    if !(3..=32).contains(&username.len()) {
        return Err(anyhow!("Username should be betwwen 3 and 32 characters"));
    }

    if !username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(anyhow!(
            "Username can only contain letters, numbers, underscores and hyphens"
        ));
    }

    Ok(())
}

pub fn validate_password(password: &str) -> Result<()> {
    if password.is_empty() {
        return Err(anyhow!("Password cannot be empty"));
    }

    if !(8..=32).contains(&password.len()) {
        return Err(anyhow!("Password should be betwwen 8 and 32 characters"));
    }

    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(anyhow!(
            "Password must contain at least one uppercase letter"
        ));
    }

    if !password.chars().any(|c| c.is_lowercase()) {
        return Err(anyhow!(
            "Password must contain at least one lowercase letter"
        ));
    }

    if !password.chars().any(|c| c.is_numeric()) {
        return Err(anyhow!("Password must contain at least one number"));
    }

    Ok(())
}