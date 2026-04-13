use anyhow::{anyhow, Result};
use bcrypt::{hash, DEFAULT_COST};
use spin_sdk::{
    http::{IntoResponse, Params, Request, Response},
    sqlite3::Value,
};

use crate::{
    models::{ChangePasswordPayload, Credentials, JsonPayload},
    util::get_connection,
};

pub(crate) fn create_account(req: Request, _params: Params) -> Result<impl IntoResponse> {
    let connection = get_connection()
        .map_err(|err| anyhow!(format!("Could not connect to the database: {}", err)))?;

    let creds = Credentials::from_request(req)?;

    connection.execute(
        "select id from Accounts where username = ?",
        &[Value::Text(creds.username.clone())],
    )?;

    if connection.changes() != 0 {
        Ok(Response::builder()
            .status(409)
            .body("username already exists")
            .build())
    } else {
        let hash = hash(creds.password, DEFAULT_COST)?;

        connection.execute(
            "insert into Accounts (username, pass_hash) values (?, ?)",
            &[Value::Text(creds.username), Value::Text(hash)],
        )?;

        Ok(Response::builder().status(200).build())
    }
}

pub(crate) fn change_password(req: Request, _params: Params) -> Result<impl IntoResponse> {
    let connection = get_connection()
        .map_err(|err| anyhow!(format!("Could not connect to the database: {}", err)))?;

    let ChangePasswordPayload {
        creds,
        new_password,
    } = ChangePasswordPayload::from_request(req)?;

    if creds.verify(&connection)? {
        let hash = hash(new_password, DEFAULT_COST)?;
        connection.execute(
            "update Accounts set pass_hash = ? where username = ?",
            &[Value::Text(hash), Value::Text(creds.username)],
        )?;

        Ok(Response::builder().status(200).build())
    } else {
        Ok(Response::builder()
            .status(400)
            .body("invalid password")
            .build())
    }
}

pub(crate) fn delete_account(req: Request, _params: Params) -> Result<impl IntoResponse> {
    let connection = get_connection()
        .map_err(|err| anyhow!(format!("Could not connect to the database: {}", err)))?;

    let creds = Credentials::from_request(req)?;

    if creds.verify(&connection)? {
        connection.execute(
            "delete from Accounts where username = ?",
            &[Value::Text(creds.username)],
        )?;
        Ok(Response::builder().status(200).build())
    } else {
        Ok(Response::builder()
            .status(400)
            .body("Invalid Username/Password")
            .build())
    }
}
