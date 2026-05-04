use anyhow::{anyhow, Result};
use bcrypt::{hash, DEFAULT_COST};
use spin_sdk::{
    http::{IntoResponse, Params, Request, Response},
};

use shared::{
    models::{ChangePasswordPayload, Credentials, JsonPayload},
    validate_password, validate_username,
};

use crate::{
    encryption::{decrypt, encrypt},
    rate_limiting::{check_rate_limit, clear_rate_limit},
    util::{FromHeader, Verify, get_connection, int, invalid_creds, rate_limit_response, text},
};

pub(crate) fn create_account(req: Request, _params: Params) -> Result<impl IntoResponse> {
    let connection =
        get_connection().map_err(|err| anyhow!("Could not connect to the database: {}", err))?;

    let creds = Credentials::from_request(req)?;
    
    validate_username(&creds.username)?;
    validate_password(&creds.password)?;

    let rows = connection
        .execute(
            "select id from Accounts where username = ?",
            &[text(&creds.username)],
        )?
        .rows;

    if rows.first().is_some() {
        Ok(Response::builder()
            .status(409)
            .body("username already exists")
            .build())
    } else {
        let hash = hash(creds.password, DEFAULT_COST)?;

        connection.execute(
            "insert into Accounts (username, pass_hash) values (?, ?)",
            &[text(&creds.username), text(&hash)],
        )?;

        Ok(Response::builder().status(201).build())
    }
}

pub(crate) fn login(req: Request, _params: Params) -> Result<impl IntoResponse> {
    let connection =
        get_connection().map_err(|err| anyhow!("Could not connect to the database: {}", err))?;

    let creds = Credentials::from_request(req)?;

    if let Err(_) = check_rate_limit(&creds.username) {
        return rate_limit_response();
    }

    if let Some(_) = creds.verify(&connection)? {
        clear_rate_limit(&creds.username)?;
        Ok(Response::builder().status(302).build())
    } else {
        Ok(Response::builder().status(401).build())
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
        return rate_limit_response();
    }

    validate_password(&new_password)?;

    if let Some(id) = creds.verify(&connection)? {
        clear_rate_limit(&creds.username)?;

        let rows = connection
            .execute(
                "select name, value from Keys where account_id = ?",
                &[int(id)],
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
                    text(&re_encrypted),
                    int(id),
                    text(&name),
                ],
            )?;
        }

        let hash = hash(new_password, DEFAULT_COST)?;
        connection.execute(
            "update Accounts set pass_hash = ? where username = ?",
            &[text(&hash), text(&creds.username)],
        )?;

        Ok(Response::builder().status(200).build())
    } else {
        invalid_creds()
    }
}

pub(crate) fn delete_account(req: Request, _params: Params) -> Result<impl IntoResponse> {
    let connection =
        get_connection().map_err(|err| anyhow!("Could not connect to the database: {}", err))?;

    let creds = Credentials::from_header(&req)?;

    if let Err(_) = check_rate_limit(&creds.username) {
        return rate_limit_response();
    }

    if let Some(id) = creds.verify(&connection)? {
        clear_rate_limit(&creds.username)?;

        connection.execute(
            "delete from Keys where account_id = ?",
            &[int(id)],
        )?;
        
        connection.execute(
            "delete from Accounts where username = ?",
            &[text(&creds.username)],
        )?;

        println!("User deleted: {}", creds.username);

        Ok(Response::builder().status(200).build())
    } else {
        invalid_creds()
    }
}
