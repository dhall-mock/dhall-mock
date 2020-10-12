use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use log::{debug, info};

use anyhow::{anyhow, Context, Error};

use super::not_found_response;
use crate::mock::service::load_configuration;
use crate::mock::service::SharedState;
use crate::web::utils;
use bytes::buf::BufExt;
use std::io::Read;

pub struct AdminServerContext {
    pub http_bind: String,
    pub state: SharedState,
}

pub(crate) async fn server(context: AdminServerContext) -> Result<(), Error> {
    let AdminServerContext { http_bind, state } = context;
    let make_svc = make_service_fn(move |_| {
        let state = state.clone();
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
    let server = Server::bind(&addr)
        .serve(make_svc)
        .with_graceful_shutdown(utils::sigint(String::from("admin service")));

    info!("Admin server started on http://{}", addr);
    server.await.context("Error on admin server execution")
}

async fn handler(req: Request<hyper::Body>, state: SharedState) -> Result<Response<Body>, Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/expectations") => {
            let read_state = state
                .read()
                .map_err(|e| anyhow!("Error acquiring lock on state : {}", e))?;

            let body = serde_json::to_string(&read_state.expectations)?;
            Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(body))
                .map_err(|_| anyhow!("Something bad happened."))
        }
        (&Method::POST, "/expectations") => {
            let mut read_body = String::new();
            hyper::body::aggregate(req)
                .await?
                .reader()
                .read_to_string(&mut read_body)?;

            match load_configuration(state, "POST web configuration".to_string(), read_body).await {
                Ok(()) => Response::builder()
                    .status(StatusCode::CREATED)
                    .body(Body::empty())
                    .map_err(|_| anyhow!("Something bad happened.")),
                Err(e) => Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::from(format!("{:#}", e)))
                    .map_err(|_| anyhow!("Something bad happened.")),
            }
        }
        _ => not_found_response(),
    }
}
