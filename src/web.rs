use log::{debug, info};
use std::convert::TryInto;
use std::sync::{Arc, RwLock};

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};

use anyhow::{anyhow, Context, Error};

use crate::expectation::model::{Expectation, HttpMethod};
use crate::mock::{look_for_expectation, IncomingRequest};

impl TryInto<HttpMethod> for &Method {
    type Error = Error;

    fn try_into(self) -> Result<HttpMethod, Self::Error> {
        match self {
            &Method::GET => Ok(HttpMethod::GET),
            &Method::POST => Ok(HttpMethod::POST),
            &Method::PUT => Ok(HttpMethod::PUT),
            &Method::DELETE => Ok(HttpMethod::DELETE),
            &Method::HEAD => Ok(HttpMethod::HEAD),
            method => Err(anyhow!("{} isn't managed as HttpMethod", method)),
        }
    }
}

impl TryInto<Response<Body>> for &Expectation {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Response<Body>, Error> {
        Response::builder()
            .status(self.response.status_code.unwrap_or(200))
            .body(
                self.response
                    .body
                    .as_ref()
                    .map(|body| Body::from(body.clone()))
                    .unwrap_or(Body::empty()),
            )
            .context(format!("Error creating http response for {:?}", self))
    }
}

pub struct State {
    pub expectations: Vec<Expectation>,
}

async fn handler<T>(req: Request<T>, state: Arc<RwLock<State>>) -> Result<Response<Body>, Error> {
    let incoming_request = req.method().try_into().map(|method| IncomingRequest {
        method,
        path: req.uri().path().to_string(),
    })?;
    let read_state = state
        .read()
        .map_err(|e| anyhow!("Error acquiring lock on state : {}", e))?;
    look_for_expectation(&read_state.expectations, incoming_request)
        .map(|expectation| expectation.try_into())
        .unwrap_or(
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("404 NotFound"))
                .map_err(|_| anyhow!("Error creating http not found response")),
        )
        .context(format!(
            "Error on creating http response for request {} {}",
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
