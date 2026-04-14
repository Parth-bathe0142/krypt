use anyhow::{anyhow, Ok, Result};
use serde_json::json;
use spin_sdk::{
    http::{IntoResponse, Params, Request, Response},
    sqlite3::Value,
};

use crate::{
    encryption::{decrypt, encrypt},
    models::{ChangeKeyPayload, Credentials, JsonPayload, KeyPayload},
    rate_limiting::{check_rate_limit, clear_rate_limit},
    util::{get_connection, invalid_creds},
};

pub(crate) fn get_key(req: Request, _param: Params) -> Result<impl IntoResponse> {
    let connection =
        get_connection().map_err(|err| anyhow!("Could not connect to the database: {}", err))?;

    let KeyPayload { creds, key } = KeyPayload::from_request(req)?;

    if let Err(_) = check_rate_limit(&creds.username) {
        return Ok(Response::builder()
            .status(429)
            .body("Too many attempts, try again later")
            .build());
    }

    if let Some(id) = creds.verify(&connection)? {
        clear_rate_limit(&creds.username)?;

        let rows = connection
            .execute(
                "select value from Keys where name = ? and account_id = ?",
                &[Value::Text(key.name), Value::Integer(id)],
            )?
            .rows;

        let value = rows
            .first()
            .ok_or(anyhow!(""))?
            .get::<&str>(0)
            .to_owned()
            .unwrap();

        let value = decrypt(&value, &creds.password, &creds.username, &connection)?;

        Ok(Response::builder()
            .status(200)
            .header("Content-Type", "text")
            .body(value)
            .build())
    } else {
        invalid_creds()
    }
}

pub(crate) fn list_keys(req: Request, _param: Params) -> Result<impl IntoResponse> {
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

        let result = connection
            .execute(
                "select name, value from Keys account_id = ?",
                &[Value::Integer(id)],
            )?
            .rows()
            .map(|row| {
                json!({
                    "name": row.get::<&str>("name").unwrap().to_owned(),
                    "value": row.get::<&str>("value").unwrap().to_owned(),
                })
            })
            .collect::<Vec<_>>();

        Ok(Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&result)?)
            .build())
    } else {
        invalid_creds()
    }
}

pub(crate) fn set_key(req: Request, _param: Params) -> Result<impl IntoResponse> {
    let connection =
        get_connection().map_err(|err| anyhow!("Could not connect to the database: {}", err))?;

    let KeyPayload { creds, key } = KeyPayload::from_request(req)?;

    if let Err(_) = check_rate_limit(&creds.username) {
        return Ok(Response::builder()
            .status(429)
            .body("Too many attempts, try again later")
            .build());
    }

    if let Some(id) = creds.verify(&connection)? {
        clear_rate_limit(&creds.username)?;

        if let Some(val) = key.value {
            let encrypted = encrypt(&val, &creds.password, &creds.username, &connection)?;
            connection.execute(
                "insert into Keys (account_id, name, value) values (?, ?, ?)",
                &[
                    Value::Integer(id),
                    Value::Text(key.name),
                    Value::Text(encrypted),
                ],
            )?;

            if connection.changes() == 1 {
                Ok(Response::builder().status(201).build())
            } else {
                Err(anyhow!("Error saving key"))
            }
        } else {
            Err(anyhow!("Missing key value"))
        }
    } else {
        invalid_creds()
    }
}

pub(crate) fn change_key(req: Request, _param: Params) -> Result<impl IntoResponse> {
    let connection =
        get_connection().map_err(|err| anyhow!("Could not connect to the database: {}", err))?;

    let ChangeKeyPayload {
        creds,
        name,
        new_value,
    } = ChangeKeyPayload::from_request(req)?;

    if let Err(_) = check_rate_limit(&creds.username) {
        return Ok(Response::builder()
            .status(429)
            .body("Too many attempts, try again later")
            .build());
    }

    if let Some(id) = creds.verify(&connection)? {
        clear_rate_limit(&creds.username)?;

        let encrypted = encrypt(&new_value, &creds.password, &creds.username, &connection)?;

        connection.execute(
            "update Keys set value = ? where account_id = ? and name = ?",
            &[
                Value::Text(encrypted),
                Value::Integer(id),
                Value::Text(name),
            ],
        )?;

        if connection.changes() == 1 {
            Ok(Response::builder().status(201).build())
        } else {
            Err(anyhow!("Key does not exist"))
        }
    } else {
        invalid_creds()
    }
}

pub(crate) fn delete_key(req: Request, _param: Params) -> Result<impl IntoResponse> {
    let connection =
        get_connection().map_err(|err| anyhow!("Could not connect to the database: {}", err))?;

    let KeyPayload { creds, key } = KeyPayload::from_request(req)?;

    if let Err(_) = check_rate_limit(&creds.username) {
        return Ok(Response::builder()
            .status(429)
            .body("Too many attempts, try again later")
            .build());
    }

    if let Some(id) = creds.verify(&connection)? {
        clear_rate_limit(&creds.username)?;

        connection.execute(
            "delete from Keys where account_id = ? and name = ?",
            &[Value::Integer(id), Value::Text(key.name)],
        )?;

        if connection.changes() == 1 {
            Ok(Response::builder().status(200).build())
        } else {
            Err(anyhow!("Error deleting key"))
        }
    } else {
        invalid_creds()
    }
}
