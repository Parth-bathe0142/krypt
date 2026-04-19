use anyhow::{anyhow, Result};
use bcrypt::{hash, DEFAULT_COST};
use spin_sdk::{
    http::{IntoResponse, Params, Request, Response},
    sqlite::Value,
};

use shared::{validate_password, validate_username};

use crate::{
    encryption::{decrypt, encrypt},
    models::{ChangePasswordPayload, Credentials, JsonPayload},
    rate_limiting::{check_rate_limit, clear_rate_limit},
    util::{get_connection, invalid_creds},
};

pub(crate) fn create_account(req: Request, _params: Params) -> Result<impl IntoResponse> {
    let connection =
        get_connection().map_err(|err| anyhow!("Could not connect to the database: {}", err))?;

    let creds = Credentials::from_request(req)?;

    let rows = connection
        .execute(
            "select id from Accounts where username = ?",
            &[Value::Text(creds.username.clone())],
        )?
        .rows;

    validate_username(&creds.username)?;
    validate_password(&creds.password)?;

    if rows.first().is_some() {
        Ok(Response::builder()
            .status(409)
            .body("username already exists")
            .build())
    } else {
        let hash = hash(creds.password, DEFAULT_COST)?;

        connection.execute(
            "insert into Accounts (username, pass_hash) values (?, ?)",
            &[Value::Text(creds.username.clone()), Value::Text(hash)],
        )?;

        connection
            .execute(
                "select id, username, pass_hash, salt from Accounts where username = ?",
                &[Value::Text(creds.username)],
            )?
            .rows()
            .next()
            .map(|row| {
                (
                    row.get::<i32>("id").unwrap(),
                    row.get::<&str>("username").unwrap(),
                    row.get::<&str>("pass_hash").unwrap(),
                    row.get::<&[u8]>("salt").unwrap(),
                )
            })
            .inspect(|(id, user, pass, salt)| println!("New user: {id}, {user}, {pass}, {salt:?}"));

        Ok(Response::builder().status(201).build())
    }
}

pub(crate) fn change_password(req: Request, _params: Params) -> Result<impl IntoResponse> {
    let connection =
        get_connection().map_err(|err| anyhow!("Could not connect to the database: {}", err))?;

    let ChangePasswordPayload {
        creds,
        new_password,
    } = ChangePasswordPayload::from_request(req)?;

    if let Err(_) = check_rate_limit(&creds.username) {
        return Ok(Response::builder()
            .status(429)
            .body("Too many attempts, try again later")
            .build());
    }

    validate_password(&creds.password)?;

    if let Some(id) = creds.verify(&connection)? {
        clear_rate_limit(&creds.username)?;

        let rows = connection
            .execute(
                "select name, value from Keys where account_id = ?",
                &[Value::Integer(id)],
            )?
            .rows;

        for row in rows {
            let name: String = row
                .get::<&str>(0)
                .ok_or(anyhow!("Missing key name"))?
                .to_owned();
            let encrypted_value = row.get::<&str>(1).ok_or(anyhow!("Missing key value"))?;

            let plaintext = decrypt(
                encrypted_value,
                &creds.password,
                &creds.username,
                &connection,
            )?;
            let re_encrypted = encrypt(&plaintext, &new_password, &creds.username, &connection)?;

            connection.execute(
                "update Keys set value = ? where account_id = ? and name = ?",
                &[
                    Value::Text(re_encrypted),
                    Value::Integer(id),
                    Value::Text(name),
                ],
            )?;
        }

        let hash = hash(new_password, DEFAULT_COST)?;
        connection.execute(
            "update Accounts set pass_hash = ? where username = ?",
            &[Value::Text(hash), Value::Text(creds.username)],
        )?;

        Ok(Response::builder().status(200).build())
    } else {
        invalid_creds()
    }
}

pub(crate) fn delete_account(req: Request, _params: Params) -> Result<impl IntoResponse> {
    let connection =
        get_connection().map_err(|err| anyhow!("Could not connect to the database: {}", err))?;

    let creds = Credentials::from_request(req)?;

    if let Err(_) = check_rate_limit(&creds.username) {
        return Ok(Response::builder()
            .status(429)
            .body("Too many attempts, try again later")
            .build());
    }

    if let Some(id) = creds.verify(&connection)? {
        clear_rate_limit(&creds.username)?;

        connection.execute(
            "delete from Accounts where username = ?",
            &[Value::Text(creds.username.clone())],
        )?;

        connection.execute(
            "delete from Keys where account_id = ?",
            &[Value::Integer(id)],
        )?;

        println!("User deleted: {}", creds.username);
        
        Ok(Response::builder().status(200).build())
    } else {
        invalid_creds()
    }
}
