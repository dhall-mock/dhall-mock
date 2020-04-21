use anyhow::{anyhow, Error};
use hyper::{Body, Response, StatusCode};

pub mod admin;
pub mod mock;

fn not_found_response() -> Result<Response<Body>, Error> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("404 NotFound"))
        .map_err(|_| anyhow!("Error creating http not found response"))
}
