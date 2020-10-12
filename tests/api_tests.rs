#![feature(async_closure)]
extern crate dhall_mock;

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::{Arc, RwLock};
use std::{fs, panic};

use anyhow::Error;
use futures::Future;
use lazy_static::lazy_static;
use reqwest::blocking::Client;

use dhall_mock::mock::model::{Expectation, HttpMethod, HttpRequest, HttpResponse};
use dhall_mock::mock::service::{load_configuration, SharedState, State};
use dhall_mock::start_servers;
use dhall_mock::web::admin::AdminServerContext;
use dhall_mock::web::mock::MockServerContext;

lazy_static! {
    static ref PORT_USED: Mutex<Vec<u16>> = Mutex::new(vec![]);
}

async fn run_server(web_port: u16, admin_port: u16, state: SharedState) -> Result<(), Error> {
    let conf = fs::read_to_string("./dhall/static.dhall").unwrap();
    load_configuration(state.clone(), "Init conf".to_string(), conf)
        .await
        .unwrap();

    start_servers(
        MockServerContext {
            http_bind: format!("0.0.0.0:{}", web_port),
            state: state.clone(),
        },
        AdminServerContext {
            http_bind: format!("0.0.0.0:{}", admin_port),
            state: state.clone(),
        },
    )
    .await
}

async fn test_wrapper<T, O>(
    test: T,
    web_port: u16,
    admin_port: u16,
    state: SharedState,
) -> Result<(), Error>
where
    T: FnOnce(SharedState, u16, u16) -> O,
    O: Future<Output = ()>,
{
    test(state, web_port, admin_port);
    Ok(())
}

async fn run_test<T, O>(web_port: u16, admin_port: u16, test: T)
where
    T: FnOnce(SharedState, u16, u16) -> O,
    O: Future<Output = ()>,
{
    let state = Arc::new(RwLock::new(State {
        expectations: vec![],
    }));

    tokio::spawn(run_server(
        web_port.clone(),
        admin_port.clone(),
        state.clone(),
    ));
    test_wrapper(test, web_port.clone(), admin_port.clone(), state.clone())
        .await
        .expect("Error running test closure");
}

#[tokio::test]
async fn test_api() {
    run_test(8000, 9000, async move |_, web_port, _| {
        let api = format!("http://{}:{}/greet/pwet", "localhost", web_port);
        let req = reqwest::blocking::get(&api).unwrap();

        assert_eq!(reqwest::StatusCode::CREATED, req.status());
    })
    .await
}

#[tokio::test]
async fn test_admin_api() {
    run_test(8001, 9001, async move |_, _, admin_port| {
        let api = format!("http://{}:{}/expectations", "localhost", admin_port);
        let req = reqwest::blocking::get(&api).unwrap();

        assert_eq!(reqwest::StatusCode::OK, req.status());
    })
    .await
}

#[tokio::test]
async fn test_admin_api_post_expectations() {
    run_test(8002, 9002, async move |state, _, admin_port| {
        let api = format!("http://{}:{}/expectations", "localhost", admin_port);
        let req = Client::builder()
            .build()
            .unwrap()
            .post(&api)
            .body(
                r#"
        let Mock = ./dhall/Mock/package.dhall
        let expectations = [
                               { request  = Mock.HttpRequest::{ method  = Some Mock.HttpMethod.GET
                                                              , path    = Some "/greet/toto"
                                                              }
                               , response = Mock.HttpResponse::{ statusCode   = Some 201
                                                               , body         = Some "Hello, toto ! Ca vient du web"
                                                               }
                              }
                           ]
        in expectations
        "#,
            )
            .send()
            .unwrap();

        assert_eq!(reqwest::StatusCode::CREATED, req.status());

        let state = state.read().unwrap();

        let expected = Expectation {
            request: HttpRequest {
                method: Some(HttpMethod::GET),
                path: Some("/greet/toto".to_string()),
                body: None,
                params: vec![],
                headers: HashMap::new(),
            },
            response: HttpResponse {
                status_code: Some(201),
                status_reason: None,
                body: Some("Hello, toto ! Ca vient du web".to_string()),
                headers: HashMap::new(),
            },
        };

        assert!(state.expectations.contains(&expected))
    }).await
}

#[tokio::test]
async fn test_admin_fail_compile_configuration() {
    run_test(8003, 9003, async move |state, _, admin_port| {
        let api = format!("http://{}:{}/expectations", "localhost", admin_port);
        let req = Client::builder()
            .build()
            .unwrap()
            .post(&api)
            .body(
                r#"
        let Mock = ./dhall/Mock/package.dhall
        let expectations = [
                               { request  = { method  = Some Mock.HttpMethod.GET
                                           , path    = Some "/greet/toto"
                                           }
                               , response = { statusCode   = Some +201
                                               , statusReason = None Text
                                               , body         = Some "Hello, toto ! Ca vient du web"
                                               }
                              }
                           ]
        in expectation
        "#,
            )
            .send()
            .unwrap();

        assert_eq!(reqwest::StatusCode::BAD_REQUEST, req.status());

        let state = state.read().unwrap();

        let expected = Expectation {
            request: HttpRequest {
                method: Some(HttpMethod::GET),
                path: Some("/greet/toto".to_string()),
                body: None,
                params: vec![],
                headers: HashMap::new(),
            },
            response: HttpResponse {
                status_code: Some(201),
                status_reason: None,
                body: Some("Hello, toto ! Ca vient du web".to_string()),
                headers: HashMap::new(),
            },
        };

        assert!(!state.expectations.contains(&expected))
    })
    .await
}
