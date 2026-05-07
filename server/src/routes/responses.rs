use anyhow::Result;
use http::StatusCode;
use spin_sdk::http::Response;

#[inline]
pub(crate) fn ok_response() -> Result<Response> {
    Ok(Response::builder().status(StatusCode::OK).build())
}

#[inline]
pub(crate) fn ok_response_with_body(body: &str, content_type: &str) -> Result<Response> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", content_type)
        .body(body)
        .build())
}

#[inline]
pub(crate) fn created_response() -> Result<Response> {
    Ok(Response::builder().status(StatusCode::CREATED).build())
}

#[inline]
pub(crate) fn not_found_response() -> Result<Response> {
    Ok(Response::builder().status(StatusCode::NOT_FOUND).build())
}

#[inline]
pub(crate) fn invalid_creds() -> Result<Response> {
    Ok(Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .body("Invalid Username/Password")
        .build())
}

#[inline]
pub(crate) fn rate_limit_response() -> Result<Response> {
    Ok(Response::builder()
        .status(StatusCode::TOO_MANY_REQUESTS)
        .body("Too many attempts, try again later")
        .build())
}
