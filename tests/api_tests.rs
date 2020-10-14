extern crate dhall_mock;

use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::Mutex;
use std::sync::{Arc, RwLock};
use std::{fs, panic};

use lazy_static::lazy_static;
use reqwest::Client;

use dhall_mock::mock::model::{Expectation, HttpMethod, HttpRequest, HttpResponse};
use dhall_mock::mock::service::{
    add_expectations_in_state, load_dhall_expectation, SharedState, State,
};
use dhall_mock::start_servers;
use dhall_mock::web::admin::AdminServerContext;
use dhall_mock::web::mock::MockServerContext;
use futures::TryFutureExt;

lazy_static! {
    static ref PORT_USED: Arc<Mutex<Vec<(u16, u16)>>> = Arc::new(Mutex::new(vec![
        (10000, 11000),
        (10001, 11001),
        (10002, 11002),
        (10003, 11003),
        (10004, 11004)
    ]));
}

async fn start_api() -> (SharedState, u16, u16) {
    let (web_port, admin_port) = PORT_USED
        .clone()
        .lock()
        .expect("Can't get lock for availables ports")
        .deref_mut()
        .pop()
        .expect("No available ports");
    let state = Arc::new(RwLock::new(State {
        expectations: vec![],
    }));
    tokio::spawn(start_servers(
        MockServerContext {
            http_bind: format!("0.0.0.0:{}", web_port),
            state: state.clone(),
        },
        AdminServerContext {
            http_bind: format!("0.0.0.0:{}", admin_port),
            state: state.clone(),
        },
    ));
    (state, web_port, admin_port)
}

#[tokio::test]
async fn test_api() {
    let (state, web_port, _) = start_api().await;

    let conf = fs::read_to_string("./dhall/static.dhall").unwrap();
    load_dhall_expectation("Init conf".to_string(), conf)
        .and_then(|expectations| add_expectations_in_state(state.clone(), expectations))
        .await
        .expect("Error loading ./dhall/static.dhall conf");

    let api = format!("http://{}:{}/greet/pwet", "localhost", web_port);
    let req = reqwest::get(&api).await.unwrap();

    assert_eq!(reqwest::StatusCode::CREATED, req.status());
}

#[tokio::test]
async fn test_admin_api() {
    let (state, _, admin_port) = start_api().await;

    let conf = fs::read_to_string("./dhall/static.dhall").unwrap();
    load_dhall_expectation("Init conf".to_string(), conf)
        .and_then(|expectations| add_expectations_in_state(state.clone(), expectations))
        .await
        .expect("Error loading ./dhall/static.dhall conf");

    let api = format!("http://{}:{}/expectations", "localhost", admin_port);
    let req = reqwest::get(&api).await.unwrap();

    assert_eq!(reqwest::StatusCode::OK, req.status());
}

#[tokio::test]
async fn test_admin_api_post_expectations() {
    let (state, _, admin_port) = start_api().await;

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
        .await
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
}

#[tokio::test]
async fn test_admin_fail_compile_configuration() {
    let (state, _, admin_port) = start_api().await;

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
        .await
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
}
