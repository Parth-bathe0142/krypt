use anyhow::Result;
use spin_sdk::http::{IntoResponse, Params, Request};

use shared::models::{ChangeKeyPayload, Credentials, JsonPayload, KeyPayload};

use crate::{
    encryption::{decrypt, encrypt},
    log,
    rate_limiting::{check_rate_limit, clear_rate_limit},
    routes::responses::{
        bad_request, conflict_response, created_response, invalid_creds, not_found_response,
        ok_response, ok_response_with_body, rate_limit_response,
    },
    util::{get_connection, int, text, FromHeader, Verify},
};

pub(crate) fn get_key(req: Request, _param: Params) -> Result<impl IntoResponse> {
    // 500
    let connection = get_connection()?;

    // BAD_REQUEST
    let Ok(KeyPayload { creds, key }) = KeyPayload::from_header(&req) else {
        return bad_request();
    };

    if let Err(_) = check_rate_limit(&creds.username) {
        log::warn(&format!("Too many requests for {}", creds.username));
        // TOO_MANY_REQUESTS
        return rate_limit_response();
    }

    // 500
    if let Some(id) = creds.verify(&connection)? {
        // 500
        clear_rate_limit(&creds.username)?;

        // 500
        let rows = connection
            .execute(
                "select value from Keys where name = ? and account_id = ?",
                &[text(&key.name), int(id)],
            )?
            .rows;

        let value = rows.first();

        if value.is_some() {
            let value = value.unwrap().get::<&str>(0).to_owned().unwrap();
            // 500
            let value = decrypt(value, &creds.password, &creds.username, &connection)?;

            // OK retrived value
            ok_response_with_body(&value, "text/plain")
        } else {
            // NOT_FOUND
            not_found_response()
        }
    } else {
        // UNAUTHORIZED invalid credentials
        invalid_creds()
    }
}

pub(crate) fn list_keys(req: Request, _param: Params) -> Result<impl IntoResponse> {
    // 500
    let connection = get_connection()?;

    // BAD_REQUEST
    let Ok(creds) = Credentials::from_header(&req) else {
        return bad_request();
    };

    if let Err(_) = check_rate_limit(&creds.username) {
        // TOO_MANY_REQUESTS
        return rate_limit_response();
    }

    // 500
    if let Some(id) = creds.verify(&connection)? {
        // 500
        clear_rate_limit(&creds.username)?;

        // 500
        let result = connection
            .execute("select name from Keys where account_id = ?", &[int(id)])?
            .rows()
            .map(|row| row.get::<&str>("name").unwrap().to_owned())
            .collect::<Vec<_>>();

        if result.is_empty() {
            // NOT_FOUND
            not_found_response()
        } else {
            // 500
            let json = serde_json::to_string(&result)?;
            // OK retrived names
            ok_response_with_body(&json, "application/json")
        }
    } else {
        // UNAUTHORIZED invalid credentials
        invalid_creds()
    }
}

pub(crate) fn set_key(req: Request, _param: Params) -> Result<impl IntoResponse> {
    // 500
    let connection = get_connection()?;

    // BAD_REQUEST
    let Ok(KeyPayload { creds, key }) = KeyPayload::from_request(req) else {
        return bad_request();
    };

    if let Err(_) = check_rate_limit(&creds.username) {
        log::warn(&format!("Too many requests for {}", creds.username));
        // TOO_MANY_REQUESTS
        return rate_limit_response();
    }

    // 500
    if let Some(id) = creds.verify(&connection)? {
        // 500
        clear_rate_limit(&creds.username)?;

        // 500
        if connection
            .execute(
                "select name from Keys where account_id = ? and name = ?",
                &[int(id), text(&key.name)],
            )?
            .rows()
            .next()
            .is_some()
        {
            // CONFLICT key already exists
            return conflict_response("Key already exists");
        }

        if let Some(val) = key.value {
            // 500
            let encrypted = encrypt(&val, &creds.password, &creds.username, &connection)?;
            // 500
            connection.execute(
                "insert into Keys (account_id, name, value) values (?, ?, ?)",
                &[int(id), text(&key.name), text(&encrypted)],
            )?;

            // CREATED new key
            created_response()
        } else {
            // BAD_REQUEST key value is missing
            bad_request()
        }
    } else {
        // UNAUTHORIZED invalid credentials
        invalid_creds()
    }
}

pub(crate) fn change_key(req: Request, _param: Params) -> Result<impl IntoResponse> {
    // 500
    let connection = get_connection()?;

    // BAD_REQUEST
    let Ok(ChangeKeyPayload {
        creds,
        name,
        new_value,
    }) = ChangeKeyPayload::from_request(req)
    else {
        return bad_request();
    };

    if let Err(_) = check_rate_limit(&creds.username) {
        log::warn(&format!("Too many requests for {}", creds.username));
        // TOO_MANY_REQUESTS
        return rate_limit_response();
    }

    // 500
    if let Some(id) = creds.verify(&connection)? {
        // 500
        clear_rate_limit(&creds.username)?;

        // 500
        if connection
            .execute(
                "select name from Keys where account_id = ? and name = ?",
                &[int(id), text(&name)],
            )?
            .rows()
            .next()
            .is_none()
        {
            // NOT_FOUND
            return not_found_response();
        }

        // 500
        let encrypted = encrypt(&new_value, &creds.password, &creds.username, &connection)?;

        // 500
        connection.execute(
            "update Keys set value = ? where account_id = ? and name = ?",
            &[text(&encrypted), int(id), text(&name)],
        )?;

        // CREATED updated value
        created_response()
    } else {
        // UNAUTHORIZED invalid credentials
        invalid_creds()
    }
}

pub(crate) fn delete_key(req: Request, _param: Params) -> Result<impl IntoResponse> {
    // 500
    let connection = get_connection()?;

    // BAD_REQUEST
    let Ok(KeyPayload { creds, key }) = KeyPayload::from_header(&req) else {
        return bad_request();
    };

    if let Err(_) = check_rate_limit(&creds.username) {
        log::warn(&format!("Too many requests for {}", creds.username));
        // TOO_MANY_REQUESTS
        return rate_limit_response();
    }

    // 500
    if let Some(id) = creds.verify(&connection)? {
        // 500
        clear_rate_limit(&creds.username)?;

        // 500
        if connection
            .execute(
                "select name from Keys where account_id = ? and name = ?",
                &[int(id), text(&key.name)],
            )?
            .rows()
            .next()
            .is_none()
        {
            // NOT_FOUND
            return not_found_response();
        }

        // 500
        connection.execute(
            "delete from Keys where account_id = ? and name = ?",
            &[int(id), text(&key.name)],
        )?;

        // OK deleted
        ok_response()
    } else {
        // UNAUTHORIZED invalid creds
        invalid_creds()
    }
}
