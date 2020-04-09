use log::{debug, info};
use std::convert::TryFrom;
use std::sync::{Arc, RwLock};

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};

use anyhow::{anyhow, Context, Error};

use crate::expectation::model::{Expectation, HttpMethod};
use crate::mock::{look_for_expectation, IncomingRequest};

impl TryFrom<&Method> for HttpMethod {
    type Error = anyhow::Error;

    fn try_from(value: &Method) -> Result<Self, Self::Error> {
        match value {
            &Method::GET => Ok(HttpMethod::GET),
            &Method::POST => Ok(HttpMethod::POST),
            &Method::PUT => Ok(HttpMethod::PUT),
            &Method::DELETE => Ok(HttpMethod::DELETE),
            &Method::HEAD => Ok(HttpMethod::HEAD),
            method => Err(anyhow!("{} isn't managed as HttpMethod", method)),
        }
    }
}

impl<T> TryFrom<&Request<T>> for IncomingRequest {
    type Error = anyhow::Error;

    fn try_from(value: &Request<T>) -> Result<Self, Self::Error> {
        Ok(IncomingRequest {
            method: HttpMethod::try_from(value.method())?,
            path: value.uri().path().to_string(),
        })
    }
}

impl TryFrom<&Expectation> for Response<Body> {
    type Error = anyhow::Error;

    fn try_from(value: &Expectation) -> Result<Response<Body>, Error> {
        Response::builder()
            .status(value.response.status_code.unwrap_or(200))
            .body(
                value
                    .response
                    .body
                    .as_ref()
                    .map(|body| Body::from(body.clone()))
                    .unwrap_or(Body::empty()),
            )
            .context(format!("Error creating http response for {:?}", value))
    }
}

pub struct State {
    pub expectations: Vec<Expectation>,
}

fn not_found_response() -> Result<Response<Body>, Error> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("404 NotFound"))
        .map_err(|_| anyhow!("Error creating http not found response"))
}

async fn handler<T>(req: Request<T>, state: Arc<RwLock<State>>) -> Result<Response<Body>, Error> {
    let read_state = state
        .read()
        .map_err(|e| anyhow!("Error acquiring lock on state : {}", e))?;

    // Try converting hyper request in IncomingRequest -> Result<IncomingRequest, Error>
    IncomingRequest::try_from(&req)
        // Convert in a Option<IncomingRequest>
        .ok()
        // Search for a matching expectation
        .and_then(|incoming_request| {
            look_for_expectation(&read_state.expectations, incoming_request)
        })
        // Convert expectation in a http response (can fail) -> Option<Result<Response<Body>, Error>>
        .map(|expectation| Response::try_from(expectation))
        // Extract the result of the option or a not found response (that can fail too) Result<Response<Body>, Error>
        .unwrap_or(not_found_response())
        // Enrich error if there is one
        .context(format!(
            "Error on handling http request {} {}",
            req.method(),
            req.uri().path()
        ))
}

pub async fn web_server(state: Arc<RwLock<State>>, http_bind: String) -> Result<(), Error> {
    let make_svc = make_service_fn(move |_| {
        let state = Arc::clone(&state);
        async {
            Ok::<_, Error>(service_fn(move |req| {
                debug!(
                    "Received http request {} on {}",
                    req.method(),
                    req.uri().path()
                );
                handler(req, state.clone())
            }))
        }
    });

    let addr = http_bind
        .parse()
        .context(format!("{} is not a valid ip config", http_bind))?;
    let server = Server::bind(&addr).serve(make_svc);

    info!("Http server started on http://{}", addr);
    server.await.context("Error on web server execution")
}
