use anyhow::anyhow;
use anyhow::Result;
use bcrypt::verify;
use spin_sdk::sqlite::Value;
use spin_sdk::{
    http::{Params, Request, Response},
    sqlite::Connection,
};

use crate::models::Credentials;

pub(crate) fn pong(_: Request, _: Params) -> Result<Response> {
    Ok(Response::builder()
        .status(200)
        .body("pong".to_string())
        .build())
}

pub(crate) fn get_connection() -> Result<Connection> {
    let connection = Connection::open("default")?;

    connection.execute(
        "create table if not exists Accounts (
            id integer primary key,
            username text unique,
            pass_hash text,
            salt BLOB NOT NULL DEFAULT (randomblob(16))
        )",
        &[],
    )?;

    connection.execute(
        "create table if not exists Keys (
            account_id integer references Accounts(id),
            name text,
            value text,
            primary key(account_id, name)
        )",
        &[],
    )?;

    Ok(connection)
}

impl Credentials {
    pub fn verify(&self, conn: &Connection) -> anyhow::Result<Option<i64>> {
        let rows = conn.execute(
            "select pass_hash, id from Accounts where username = ?",
            &[Value::Text(self.username.clone())],
        )?;

        let Some(row) = rows.rows.first() else {
            return Ok(None);
        };

        let pass_hash = row.get::<&str>(0).unwrap();
        if verify(&self.password, pass_hash).map_err(|_| anyhow!("Error verifying password"))? {
            Ok(row.get(1))
        } else {
            Ok(None)
        }
    }
}

#[inline]
pub(crate) fn invalid_creds() -> Result<Response> {
    Ok(Response::builder()
        .status(400)
        .body("Invalid Username/Password")
        .build())
}

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

pub(crate) fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub(crate) fn get_salt(username: &str, conn: &Connection) -> Result<String> {
    Ok(conn
        .execute(
            "select salt from Accounts where username = ?",
            &[Value::Text(username.to_owned())],
        )?
        .rows
        .first()
        .unwrap()
        .get::<&str>(0)
        .map(ToString::to_string)
        .unwrap())
}
