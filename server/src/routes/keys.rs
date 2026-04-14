use anyhow::{anyhow, Ok, Result};
use spin_sdk::{
    http::{IntoResponse, Params, Request, Response},
    sqlite3::Value,
};

use crate::{
    models::{ChangeKeyPayload, JsonPayload, KeyPayload},
    util::{get_connection, invalid_creds},
};

pub(crate) fn get_key(req: Request, _param: Params) -> Result<impl IntoResponse> {
    let connection =
        get_connection().map_err(|err| anyhow!("Could not connect to the database: {}", err))?;

    let KeyPayload { creds, key } = KeyPayload::from_request(req)?;

    if creds.verify(&connection)? {
        let id = connection
            .execute(
                "select id from Accounts where username = ?",
                &[Value::Text(creds.username)],
            )?
            .rows
            .first()
            .ok_or(anyhow!("Error fetching account"))?
            .get(0)
            .unwrap();

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

        Ok(Response::builder().status(200).body(value).build())
    } else {
        invalid_creds()
    }
}

pub(crate) fn set_key(req: Request, _param: Params) -> Result<impl IntoResponse> {
    let connection =
        get_connection().map_err(|err| anyhow!("Could not connect to the database: {}", err))?;

    let KeyPayload { creds, key } = KeyPayload::from_request(req)?;

    if creds.verify(&connection)? {
        let id = connection
            .execute(
                "select id from Accounts where username = ?",
                &[Value::Text(creds.username)],
            )?
            .rows
            .first()
            .ok_or(anyhow!("Error fetching account"))?
            .get(0)
            .unwrap();

        if let Some(val) = key.value {
            connection.execute(
                "insert into Keys (account_id, name, value) values (?, ?, ?)",
                &[Value::Integer(id), Value::Text(key.name), Value::Text(val)],
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

    if creds.verify(&connection)? {
        let id = connection
            .execute(
                "select id from Accounts where username = ?",
                &[Value::Text(creds.username)],
            )?
            .rows
            .first()
            .ok_or(anyhow!("Error fetching account"))?
            .get(0)
            .unwrap();

        connection
            .execute(
                "select * from Keys where account_id = ? and name = ?",
                &[Value::Integer(id), Value::Text(name.clone())],
            )?
            .rows
            .first()
            .ok_or(anyhow!("Key does not exist"))?;

        connection.execute(
            "update Keys set value = ? where account_id = ? and name = ?",
            &[
                Value::Text(new_value),
                Value::Integer(id),
                Value::Text(name),
            ],
        )?;

        if connection.changes() == 1 {
            Ok(Response::builder().status(201).build())
        } else {
            Err(anyhow!("Error changing key value"))
        }
    } else {
        invalid_creds()
    }
}

pub(crate) fn delete_key(req: Request, _param: Params) -> Result<impl IntoResponse> {
    let connection =
        get_connection().map_err(|err| anyhow!("Could not connect to the database: {}", err))?;

    let KeyPayload { creds, key } = KeyPayload::from_request(req)?;

    if creds.verify(&connection)? {
        let id = connection
            .execute(
                "select id from Accounts where username = ?",
                &[Value::Text(creds.username)],
            )?
            .rows
            .first()
            .ok_or(anyhow!("Error fetching account"))?
            .get(0)
            .unwrap();

        if let Some(val) = key.value {
            connection
                .execute(
                    "select * from Keys where account_id = ? and name = ? and value = ?",
                    &[
                        Value::Integer(id),
                        Value::Text(key.name.clone()),
                        Value::Text(val),
                    ],
                )?
                .rows
                .first()
                .ok_or(anyhow!("Error fetching account"))?;

            connection.execute(
                "delete from Keys where id = ? and name = ?",
                &[Value::Integer(id), Value::Text(key.name)],
            )?;

            if connection.changes() == 1 {
                Ok(Response::builder().status(200).build())
            } else {
                Err(anyhow!("Error deleting key"))
            }
        } else {
            Err(anyhow!("Missing key value"))
        }
    } else {
        invalid_creds()
    }
}
