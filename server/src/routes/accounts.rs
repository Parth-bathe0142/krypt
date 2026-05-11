use anyhow::{anyhow, Result};
use bcrypt::{hash, DEFAULT_COST};
use spin_sdk::http::{IntoResponse, Params, Request};

use shared::{
    models::{ChangePasswordPayload, Credentials, JsonPayload},
    validate_password, validate_username,
};

use crate::{
    encryption::{decrypt, encrypt},
    log,
    rate_limiting::{check_rate_limit, clear_rate_limit},
    routes::responses::{
        accepted_response, bad_request, conflict_response, created_response, invalid_creds,
        invalid_password, invalid_username, ok_response, rate_limit_response,
    },
    util::{get_connection, int, text, FromHeader, Verify},
};

pub(crate) fn create_account(req: Request, _params: Params) -> Result<impl IntoResponse> {
    // 500
    let connection = get_connection()?;

    // BAD_REQUEST
    let Ok(creds) = Credentials::from_request(req) else {
        return bad_request();
    };

    // NOT_ACCEPTABLE
    let Ok(_) = validate_username(&creds.username) else {
        return invalid_username();
    };
    // NOT_ACCEPTABLE
    let Ok(_) = validate_password(&creds.password) else {
        return invalid_password();
    };

    // 500
    let rows = connection
        .execute(
            "select id from Accounts where username = ?",
            &[text(&creds.username)],
        )?
        .rows;

    // CONFLICT username exists
    if rows.first().is_some() {
        conflict_response("Username already exists")
    } else {
        // 500
        let hash = hash(creds.password, DEFAULT_COST)?;

        // 500
        connection.execute(
            "insert into Accounts (username, pass_hash) values (?, ?)",
            &[text(&creds.username), text(&hash)],
        )?;

        // 500
        // 500
        let id = connection
            .execute(
                "select id from Accounts where username = ?",
                &[text(&creds.username)],
            )?
            .rows()
            .next()
            .ok_or_else(|| anyhow!("failed to insert new account"))?
            .get::<i64>("id")
            .unwrap();

        log::info(&format!("Account created, id: {}", id));

        // CREATED new account
        created_response()
    }
}

pub(crate) fn login(req: Request, _params: Params) -> Result<impl IntoResponse> {
    // 500
    let connection = get_connection()?;

    // BAD_REQUEST
    let Ok(creds) = Credentials::from_request(req) else {
        return bad_request();
    };

    if let Err(_) = check_rate_limit(&creds.username) {
        log::warn(&format!("Too many requests for {}", creds.username));
        // TOO_MANY_REQUESTS
        return rate_limit_response();
    }

    // 500
    if let Some(_) = creds.verify(&connection)? {
        // 500
        clear_rate_limit(&creds.username)?;
        // ACCEPTED valid credentials
        accepted_response()
    } else {
        // UNAUTHORIZED invalid credentials
        invalid_creds()
    }
}

pub(crate) fn change_password(req: Request, _params: Params) -> Result<impl IntoResponse> {
    // 500
    let connection = get_connection()?;

    // BAD_REQUEST
    let Ok(ChangePasswordPayload {
        creds,
        new_password,
    }) = ChangePasswordPayload::from_request(req)
    else {
        return bad_request();
    };

    if let Err(_) = check_rate_limit(&creds.username) {
        log::warn(&format!("Too many requests for {}", creds.username));
        // TOO_MANY_REQUESTS
        return rate_limit_response();
    }

    // NOT_ACCEPTABLE
    let Ok(_) = validate_password(&new_password) else {
        return invalid_password();
    };

    // 500
    if let Some(id) = creds.verify(&connection)? {
        // 500
        clear_rate_limit(&creds.username)?;

        // 500
        let rows = connection
            .execute(
                "select name, value from Keys where account_id = ?",
                &[int(id)],
            )?
            .rows;

        for row in rows {
            // 500 unreachable
            let name: String = row
                .get::<&str>(0)
                .ok_or(anyhow!("Missing key name"))?
                .to_owned();

            // 500 unreachable
            let encrypted_value = row.get::<&str>(1).ok_or(anyhow!("Missing key value"))?;

            // 500
            let plaintext = decrypt(
                encrypted_value,
                &creds.password,
                &creds.username,
                &connection,
            )?;
            // 500
            let re_encrypted = encrypt(&plaintext, &new_password, &creds.username, &connection)?;

            // 500
            connection.execute(
                "update Keys set value = ? where account_id = ? and name = ?",
                &[text(&re_encrypted), int(id), text(&name)],
            )?;
        }

        // 500
        let hash = hash(new_password, DEFAULT_COST)?;

        // 500
        connection.execute(
            "update Accounts set pass_hash = ? where username = ?",
            &[text(&hash), text(&creds.username)],
        )?;

        // OK password changed
        ok_response()
    } else {
        // UNAUTHORIZED invalid credentials
        invalid_creds()
    }
}

pub(crate) fn delete_account(req: Request, _params: Params) -> Result<impl IntoResponse> {
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
        connection.execute(
            "delete from Accounts where username = ?",
            &[text(&creds.username)],
        )?;

        log::info(&format!("Account deleted, id: {}", id));

        // OK account deleted
        ok_response()
    } else {
        // UNAUTHORIZED invalid credentials
        invalid_creds()
    }
}
