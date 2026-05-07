use anyhow::{Ok, Result};
use http::StatusCode;
use spin_sdk::http::{IntoResponse, Params, Request, Response};

use shared::models::{ChangeKeyPayload, Credentials, JsonPayload, KeyPayload};

use crate::{
    encryption::{decrypt, encrypt},
    log,
    rate_limiting::{check_rate_limit, clear_rate_limit},
    routes::responses::{
        created_response, invalid_creds, not_found_response, ok_response, ok_response_with_body,
        rate_limit_response,
    },
    util::{get_connection, int, text, FromHeader, Verify},
};

pub(crate) fn get_key(req: Request, _param: Params) -> Result<impl IntoResponse> {
    let connection = get_connection()?;

    let KeyPayload { creds, key } = KeyPayload::from_header(&req)?;

    if let Err(_) = check_rate_limit(&creds.username) {
        log::warn(&format!("Too many requests for {}", creds.username));
        return rate_limit_response();
    }

    if let Some(id) = creds.verify(&connection)? {
        clear_rate_limit(&creds.username)?;

        let rows = connection
            .execute(
                "select value from Keys where name = ? and account_id = ?",
                &[text(&key.name), int(id)],
            )?
            .rows;

        let value = rows.first();

        if value.is_some() {
            let value = value.unwrap().get::<&str>(0).to_owned().unwrap();
            let value = decrypt(value, &creds.password, &creds.username, &connection)?;

            ok_response_with_body(&value, "text/plain")
        } else {
            not_found_response()
        }
    } else {
        invalid_creds()
    }
}

pub(crate) fn list_keys(req: Request, _param: Params) -> Result<impl IntoResponse> {
    let connection = get_connection()?;

    let creds = Credentials::from_header(&req)?;

    if let Err(_) = check_rate_limit(&creds.username) {
        return rate_limit_response();
    }

    if let Some(id) = creds.verify(&connection)? {
        clear_rate_limit(&creds.username)?;

        let result = connection
            .execute("select name from Keys where account_id = ?", &[int(id)])?
            .rows()
            .map(|row| row.get::<&str>("name").unwrap().to_owned())
            .collect::<Vec<_>>();

        if result.is_empty() {
            not_found_response()
        } else {
            let json = serde_json::to_string(&result)?;
            ok_response_with_body(&json, "application/json")
        }
    } else {
        invalid_creds()
    }
}

pub(crate) fn set_key(req: Request, _param: Params) -> Result<impl IntoResponse> {
    let connection = get_connection()?;

    let KeyPayload { creds, key } = KeyPayload::from_request(req)?;

    if let Err(_) = check_rate_limit(&creds.username) {
        log::warn(&format!("Too many requests for {}", creds.username));
        return rate_limit_response();
    }

    if let Some(id) = creds.verify(&connection)? {
        clear_rate_limit(&creds.username)?;

        if connection
            .execute(
                "select name from Keys where account_id = ? and name = ?",
                &[int(id), text(&key.name)],
            )?
            .rows()
            .next()
            .is_some()
        {
            return Ok(Response::builder().status(StatusCode::CONFLICT).build());
        }

        if let Some(val) = key.value {
            let encrypted = encrypt(&val, &creds.password, &creds.username, &connection)?;
            connection.execute(
                "insert into Keys (account_id, name, value) values (?, ?, ?)",
                &[int(id), text(&key.name), text(&encrypted)],
            )?;

            created_response()
        } else {
            Ok(Response::builder().status(StatusCode::BAD_REQUEST).build())
        }
    } else {
        invalid_creds()
    }
}

pub(crate) fn change_key(req: Request, _param: Params) -> Result<impl IntoResponse> {
    let connection = get_connection()?;

    let ChangeKeyPayload {
        creds,
        name,
        new_value,
    } = ChangeKeyPayload::from_request(req)?;

    if let Err(_) = check_rate_limit(&creds.username) {
        log::warn(&format!("Too many requests for {}", creds.username));
        return rate_limit_response();
    }

    if let Some(id) = creds.verify(&connection)? {
        clear_rate_limit(&creds.username)?;

        if connection
            .execute(
                "select name from Keys where account_id = ? and name = ?",
                &[int(id), text(&name)],
            )?
            .rows()
            .next()
            .is_none()
        {
            return not_found_response();
        }

        let encrypted = encrypt(&new_value, &creds.password, &creds.username, &connection)?;

        connection.execute(
            "update Keys set value = ? where account_id = ? and name = ?",
            &[text(&encrypted), int(id), text(&name)],
        )?;

        created_response()
    } else {
        invalid_creds()
    }
}

pub(crate) fn delete_key(req: Request, _param: Params) -> Result<impl IntoResponse> {
    let connection = get_connection()?;

    let KeyPayload { creds, key } = KeyPayload::from_header(&req)?;

    if let Err(_) = check_rate_limit(&creds.username) {
        log::warn(&format!("Too many requests for {}", creds.username));
        return rate_limit_response();
    }

    if let Some(id) = creds.verify(&connection)? {
        clear_rate_limit(&creds.username)?;

        if connection
            .execute(
                "select name from Keys where account_id = ? and name = ?",
                &[int(id), text(&key.name)],
            )?
            .rows()
            .next()
            .is_none()
        {
            return not_found_response();
        }

        connection.execute(
            "delete from Keys where account_id = ? and name = ?",
            &[int(id), text(&key.name)],
        )?;

        ok_response()
    } else {
        invalid_creds()
    }
}
