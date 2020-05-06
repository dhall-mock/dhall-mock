extern crate dhall_mock;

use get_port::{get_port_in_range, PortRange};
use lazy_static::lazy_static;
use reqwest::blocking::Client;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::{Arc, RwLock};
use std::{fs, panic};
use tokio::runtime;
use tokio::sync::oneshot;

use dhall_mock::mock::model::{Expectation, HttpMethod, HttpRequest, HttpResponse};
use dhall_mock::mock::service::{add_configuration, State};
use dhall_mock::start_servers;
use dhall_mock::web::admin::AdminServerContext;
use dhall_mock::web::mock::MockServerContext;

lazy_static! {
    static ref PORT_USED: Mutex<Vec<u16>> = Mutex::new(vec![]);
}

fn run_test<T>(test: T) -> ()
where
    T: FnOnce(Arc<RwLock<State>>, u16, u16) -> () + panic::UnwindSafe,
{
    let _ignore = dhall_mock::start_logger();
    let mut loader_rt = runtime::Runtime::new().unwrap();
    let mut web_rt = runtime::Runtime::new().unwrap();
    let (web_send_close, web_close_channel) = oneshot::channel::<()>();
    let (admin_send_close, admin_close_channel) = oneshot::channel::<()>();

    let state = Arc::new(RwLock::new(State {
        expectations: vec![],
    }));
    let lock = PORT_USED.lock().unwrap();

    let web_port = get_port_in_range(PortRange {
        min: 8000,
        max: 9000,
    })
    .unwrap();
    let admin_port = get_port_in_range(PortRange {
        min: 9000,
        max: 10000,
    })
    .unwrap();

    let conf = fs::read_to_string("./dhall/static.dhall").unwrap();
    loader_rt
        .block_on(async {
            tokio::task::spawn(add_configuration(
                state.clone(),
                "Init conf".to_string(),
                conf,
            ))
            .await?
        })
        .unwrap();

    let join = web_rt.spawn(start_servers(
        MockServerContext {
            http_bind: format!("0.0.0.0:{}", web_port),
            state: state.clone(),
            close_channel: web_close_channel,
        },
        AdminServerContext {
            http_bind: format!("0.0.0.0:{}", admin_port),
            state: state.clone(),
            close_channel: admin_close_channel,
            target_runtime: Arc::new(loader_rt),
        },
    ));
    drop(lock);

    let result = panic::catch_unwind(|| test(state.clone(), web_port, admin_port));

    teardown(web_send_close, admin_send_close);

    let _ = web_rt.block_on(join);
    assert!(result.is_ok())
}

fn teardown(web_send_close: oneshot::Sender<()>, admin_send_close: oneshot::Sender<()>) {
    web_send_close.send(()).unwrap();
    admin_send_close.send(()).unwrap()
}

#[test]
fn test_api() {
    run_test(|_, web_port, _| {
        let api = format!("http://{}:{}/greet/pwet", "localhost", web_port);
        let req = reqwest::blocking::get(&api).unwrap();

        assert_eq!(reqwest::StatusCode::CREATED, req.status());
    })
}

#[test]
fn test_admin_api() {
    run_test(|_, _, admin_port| {
        let api = format!("http://{}:{}/expectations", "localhost", admin_port);
        let req = reqwest::blocking::get(&api).unwrap();

        assert_eq!(reqwest::StatusCode::OK, req.status());
    })
}

#[test]
fn test_admin_api_post_expectations() {
    run_test(|state, _, admin_port| {
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
    })
}

#[test]
fn test_admin_fail_compile_configuration() {
    run_test(|state, _, admin_port| {
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
}
