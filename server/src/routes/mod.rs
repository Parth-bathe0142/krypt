use anyhow::Result;
use http::StatusCode;
use spin_sdk::http::{Params, Request, Response};

pub mod accounts;
pub(crate) use accounts::*;

pub mod keys;
pub(crate) use keys::*;

mod responses;

pub(crate) fn pong(_: Request, _: Params) -> Result<Response> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body("pong".to_string())
        .build())
}
