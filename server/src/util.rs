use anyhow::anyhow;
use anyhow::Result;
use bcrypt::verify;
use shared::models::Credentials;
use shared::models::Key;
use shared::models::KeyPayload;
use spin_sdk::sqlite::Value;
use spin_sdk::{http::Request, sqlite::Connection};

use crate::log;

fn connection() -> Result<Connection> {
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
            account_id integer references Accounts(id) ON DELETE CASCADE,
            name text,
            value text,
            primary key(account_id, name)
        )",
        &[],
    )?;

    Ok(connection)
}

pub fn get_connection() -> Result<Connection> {
    connection().map_err(|err| {
        log::error(&format!("Could not connect to the database: {}", err));
        anyhow!("Could not connect to the database: {}", err)
    })
}

pub trait Verify {
    fn verify(&self, conn: &Connection) -> anyhow::Result<Option<i64>>;
}

impl Verify for shared::models::Credentials {
    fn verify(&self, conn: &Connection) -> anyhow::Result<Option<i64>> {
        let rows = conn.execute(
            "select pass_hash, id from Accounts where username = ?",
            &[text(&self.username)],
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

pub(crate) fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub trait FromHeader: Sized {
    fn from_header(request: &Request) -> Result<Self>;
}

fn get_header(request: &Request, name: &str) -> Result<String> {
    request
        .header(name)
        .ok_or_else(|| anyhow!("Missing {} in header", name))?
        .as_str()
        .ok_or_else(|| anyhow!("Failed to parse header {}", name))
        .map(|s| s.to_owned())
}

impl FromHeader for Credentials {
    fn from_header(request: &Request) -> Result<Self> {
        let username = get_header(request, "username")?;
        let password = get_header(request, "password")?;

        Ok(Self { username, password })
    }
}

impl FromHeader for KeyPayload {
    fn from_header(request: &Request) -> Result<Self> {
        let username = get_header(request, "username")?;
        let password = get_header(request, "password")?;
        let key = get_header(request, "key")?;

        Ok(Self {
            creds: Credentials { username, password },
            key: Key {
                name: key,
                value: None,
            },
        })
    }
}

pub(crate) fn int(num: i64) -> Value {
    Value::Integer(num)
}

pub(crate) fn text(str: &str) -> Value {
    Value::Text(str.to_owned())
}
