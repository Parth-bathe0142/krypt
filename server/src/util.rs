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

pub trait Verify {
    fn verify(&self, conn: &Connection) -> anyhow::Result<Option<i64>>;
}

impl Verify for shared::models::Credentials {
    fn verify(&self, conn: &Connection) -> anyhow::Result<Option<i64>> {
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

pub(crate) fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub(crate) fn get_salt(username: &str, conn: &Connection) -> Result<Vec<u8>> {
    Ok(conn
        .execute(
            "select salt from Accounts where username = ?",
            &[Value::Text(username.to_owned())],
        )?
        .rows
        .first()
        .unwrap()
        .get::<&[u8]>(0)
        .map(ToOwned::to_owned)
        .unwrap())
}
