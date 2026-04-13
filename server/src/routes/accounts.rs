use anyhow::{anyhow, Result};
use bcrypt::{hash, DEFAULT_COST};
use spin_sdk::{
    http::{IntoResponse, Params, Request, Response},
    sqlite3::Value,
};

use crate::{
    models::{ChangePasswordPayload, Credentials},
    util::get_connection,
};

pub(crate) fn create_account(req: Request, _params: Params) -> Result<impl IntoResponse> {
    let connection = match get_connection() {
        Ok(conn) => conn,
        Err(err) => {
            return Err(anyhow!(format!(
                "Could not connect to the database: {}",
                err
            )))
        }
    };

    let Ok(creds) = String::from_utf8(req.into_body()) else {
        return Err(anyhow!("request missing body".to_string()));
    };
    let Ok(creds) = serde_json::from_str::<Credentials>(&creds) else {
        return Err(anyhow!("Could not parse request body".to_string()));
    };

    let result = connection.execute(
        "select id from Accounts where username = ?",
        &[Value::Text(creds.username.clone())],
    )?;

    if result.rows.len() != 0 {
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
    let connection = match get_connection() {
        Ok(conn) => conn,
        Err(err) => {
            return Err(anyhow!(format!(
                "Could not connect to the database: {}",
                err
            )))
        }
    };

    let Ok(creds) = String::from_utf8(req.into_body()) else {
        return Err(anyhow!("request missing body".to_string()));
    };
    
    let Ok(ChangePasswordPayload {
        creds,
        new_password,
    }) = serde_json::from_str::<ChangePasswordPayload>(&creds)
    else {
        return Err(anyhow!("Could not parse request body".to_string()));
    };
    
    if creds.verify(&connection)? {
        let hash = hash(new_password, DEFAULT_COST)?;
        connection.execute("update Accounts set pass_hash = ? where username = ?", &[Value::Text(hash), Value::Text(creds.username)])?;
        
        Ok(Response::builder().status(200).build())
    } else {
        Ok(Response::builder().status(400).body("invalid password").build())
    }
}


pub(crate) fn delete_account(req: Request, _params: Params) -> Result<impl IntoResponse> {
    let connection = match get_connection() {
        Ok(conn) => conn,
        Err(err) => {
            return Err(anyhow!(format!(
                "Could not connect to the database: {}",
                err
            )))
        }
    };

    let Ok(creds) = String::from_utf8(req.into_body()) else {
        return Err(anyhow!("request missing body".to_string()));
    };
    let Ok(creds) = serde_json::from_str::<Credentials>(&creds) else {
        return Err(anyhow!("Could not parse request body".to_string()));
    };

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