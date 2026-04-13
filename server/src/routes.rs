use anyhow::anyhow;
use bcrypt::{hash, DEFAULT_COST};
use spin_sdk::{
    http::{IntoResponse, Params, Request, Response},
    sqlite3::Value,
};

use crate::{models::Credentials, util::get_connection};

pub(crate) fn create_account(req: Request, _params: Params) -> anyhow::Result<impl IntoResponse> {
    let Ok(connection) = get_connection() else {
        return Err(anyhow!("Could not connect to the database".to_string()));
    };

    let Ok(creds) = String::from_utf8(req.into_body()) else {
        return Err(anyhow!("request missing body".to_string()));
    };
    let Ok(creds) = serde_json::from_str::<Credentials>(&creds) else {
        return Err(anyhow!("Could not parse request body".to_string()));
    };

    let result = connection
        .execute(
            "select id from Accounts where username = ?",
            &[Value::Text(creds.username.clone())],
        )?;

    if result.rows.len() != 0 {
        Ok(Response::builder().status(409).body("username already exists").build())
    } else {
        let hash = hash(creds.password, DEFAULT_COST)?;

        connection
            .execute(
                "insert into Accounts (username, pass_hash) values (?, ?)",
                &[Value::Text(creds.username), Value::Text(hash)],
            )?;

        Ok(Response::builder().status(200).build())
    }
}