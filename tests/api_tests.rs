extern crate dhall_mock;

use dhall_mock::mock::model::{Expectation, HttpMethod, HttpRequest, HttpResponse};
use dhall_mock::mock::service::{add_configuration, State};
use dhall_mock::start_servers;
use std::sync::{Arc, RwLock};
use std::{fs, panic};
use tokio::runtime;
use tokio::sync::oneshot;

use dhall_mock::web::admin::AdminServerContext;
use dhall_mock::web::mock::MockServerContext;
use reqwest::blocking::Client;
use std::net::TcpListener;

fn get_available_port(bind: &str, excluded: Option<u16>) -> Option<u16> {
    (30000..40000)
        .filter(|port| excluded != Some(*port))
        .find(|port| TcpListener::bind((bind, *port)).is_ok())
}

fn search_and_lock_available_port(bind: &str) -> Option<(TcpListener, u16)> {
    (30000..40000)
        .filter_map(|port| {
            TcpListener::bind((bind, port))
                .map(|socket| (socket, port))
                .ok()
        })
        .next()
}

struct TestContext {
    mock_bind: String,
    admin_bind: String,
}

fn run_test<T>(test: T) -> ()
where
    T: FnOnce(TestContext, Arc<RwLock<State>>) -> () + panic::UnwindSafe,
{
    let mut loader_rt = runtime::Runtime::new().unwrap();
    let mut web_rt = runtime::Runtime::new().unwrap();
    let (web_send_close, web_close_channel) = oneshot::channel::<()>();
    let (admin_send_close, admin_close_channel) = oneshot::channel::<()>();

    let state = Arc::new(RwLock::new(State {
        expectations: vec![],
    }));

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

    let mock_port = get_available_port("127.0.0.1", None)
        .expect("No port available to start mock server for test");
    let admin_port = get_available_port("127.0.0.1", Some(mock_port))
        .expect("No port available to start admin server for test");
    let mock_bind = format!("127.0.0.1:{}", mock_port);
    let admin_bind = format!("127.0.0.1:{}", admin_port);

    let host_bind = "127.0.0.1";

    let join = {
        let (_mock_lock, mock_port) = search_and_lock_available_port(host_bind)
            .expect("No port available to start mock server for test");
        let (_admin_lock, admin_port) = search_and_lock_available_port(host_bind)
            .expect("No port available to start mock server for test");
        println!("Start servers on {}, {}", mock_bind, admin_bind);
        web_rt.spawn(start_servers(
            MockServerContext {
                http_bind: format!("{}:{}", host_bind, mock_port),
                state: state.clone(),
                close_channel: web_close_channel,
            },
            AdminServerContext {
                http_bind: format!("{}:{}", host_bind, admin_port),
                state: state.clone(),
                close_channel: admin_close_channel,
                target_runtime: Arc::new(loader_rt),
            },
        ))
    };

    let result = panic::catch_unwind(|| {
        test(
            TestContext {
                mock_bind,
                admin_bind,
            },
            state.clone(),
        )
    });

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
    run_test(|test_context, _| {
        let api = format!("http://{}/greet/pwet", test_context.mock_bind);
        let req = reqwest::blocking::get(&api).unwrap();

        assert_eq!(reqwest::StatusCode::CREATED, req.status());
    })
}

#[test]
fn test_admin_api() {
    run_test(|test_context, _| {
        let api = format!("http://{}/expectations", test_context.admin_bind);
        let req = reqwest::blocking::get(&api).unwrap();

        assert_eq!(reqwest::StatusCode::OK, req.status());
    })
}

#[test]
fn test_admin_api_post_expectations() {
    run_test(|test_context, state| {
        let api = format!("http://{}/expectations", test_context.admin_bind);
        let req = Client::builder().build().unwrap().post(&api).body(r#"
        let Mock = https://raw.githubusercontent.com/dhall-mock/dhall-mock/master/dhall/Mock/package.dhall
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
        in expectations
        "#).send().unwrap();

        assert_eq!(reqwest::StatusCode::CREATED, req.status());

        let state = state.read().unwrap();

        let expected = Expectation {
            request: HttpRequest {
                method: Some(HttpMethod::GET),
                path: Some("/greet/toto".to_string()),
            },
            response: HttpResponse {
                status_code: Some(201),
                status_reason: None,
                body: Some("Hello, toto ! Ca vient du web".to_string()),
            },
        };

        assert!(state.expectations.contains(&expected))
    })
}

#[test]
fn test_admin_fail_compile_configuration() {
    run_test(|test_context, state| {
        let api = format!("http://{}/expectations", test_context.admin_bind);
        let req = Client::builder().build().unwrap().post(&api).body(r#"
        let Mock = https://raw.githubusercontent.com/dhall-mock/dhall-mock/master/dhall/Mock/package.dhall
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
        "#).send().unwrap();

        assert_eq!(reqwest::StatusCode::BAD_REQUEST, req.status());

        let state = state.read().unwrap();

        let expected = Expectation {
            request: HttpRequest {
                method: Some(HttpMethod::GET),
                path: Some("/greet/toto".to_string()),
            },
            response: HttpResponse {
                status_code: Some(201),
                status_reason: None,
                body: Some("Hello, toto ! Ca vient du web".to_string()),
            },
        };

        assert!(!state.expectations.contains(&expected))
    })
}
