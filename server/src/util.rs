use spin_sdk::{
    http::{Params, Request, Response},
    sqlite3::Connection,
};

pub(crate) fn pong(_: Request, _: Params) -> anyhow::Result<Response> {
    Ok(Response::builder()
        .status(200)
        .body("pong".to_string())
        .build())
}

pub(crate) fn get_connection() -> anyhow::Result<Connection> {
    let connection = Connection::open("default")?;
    connection.execute(
        "create table if not exists Accounts (
            id integer primary key,
            username text unique,
            pass_hash text
        )",
        &[],
    )?;

    Ok(connection)
}
