extern crate dhall_mock;

use dhall_mock::expectation::model::{Expectation, HttpMethod, HttpRequest, HttpResponse};
use dhall_mock::State;
use std::panic;
use std::sync::{Arc, RwLock};
use tokio::runtime;
use tokio::sync::oneshot;

use reqwest::blocking::Client;

fn run_test<T>(test: T) -> ()
where
    T: FnOnce(Arc<RwLock<State>>) -> () + panic::UnwindSafe,
{
    let loader_rt = Arc::new(runtime::Runtime::new().unwrap());
    let mut web_rt = runtime::Runtime::new().unwrap();
    let (web_send_close, web_close_channel) = oneshot::channel::<()>();
    let (admin_send_close, admin_close_channel) = oneshot::channel::<()>();

    let conf = dhall_mock::compiler::load_file("./dhall/static.dhall").unwrap();
    let expectations = dhall_mock::compiler::compile_configuration(conf.as_ref()).unwrap();

    let state = Arc::new(RwLock::new(dhall_mock::State {
        expectations: expectations,
    }));

    let join = setup(
        &web_rt,
        loader_rt,
        state.clone(),
        web_close_channel,
        admin_close_channel,
    );

    let result = panic::catch_unwind(|| test(state.clone()));

    teardown(web_send_close, admin_send_close);

    let _ = web_rt.block_on(join);
    assert!(result.is_ok())
}

fn setup(
    web_rt: &runtime::Runtime,
    loader_rt: Arc<runtime::Runtime>,
    state: Arc<RwLock<State>>,
    web_close_channel: oneshot::Receiver<()>,
    admin_close_channel: oneshot::Receiver<()>,
) -> tokio::task::JoinHandle<Result<(), anyhow::Error>> {
    web_rt.spawn(dhall_mock::run_mock_server(
        String::from("0.0.0.0:8088"),
        String::from("0.0.0.0:8089"),
        state,
        loader_rt,
        web_close_channel,
        admin_close_channel,
    ))
}

fn teardown(web_send_close: oneshot::Sender<()>, admin_send_close: oneshot::Sender<()>) {
    web_send_close.send(()).unwrap();
    admin_send_close.send(()).unwrap()
}

#[test]
fn test_api() {
    run_test(|_| {
        let api = format!("http://{}:{}/greet/pwet", "localhost", 8088);
        let req = reqwest::blocking::get(&api).unwrap();

        assert_eq!(reqwest::StatusCode::CREATED, req.status());
    })
}

#[test]
fn test_admin_api() {
    run_test(|_| {
        let api = format!("http://{}:{}/expectations", "localhost", 8089);
        let req = reqwest::blocking::get(&api).unwrap();

        assert_eq!(reqwest::StatusCode::OK, req.status());
    })
}

#[test]
fn test_admin_api_post_expectations() {
    run_test(|state| {
        let api = format!("http://{}:{}/expectations", "localhost", 8089);
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
    run_test(|state| {
        let api = format!("http://{}:{}/expectations", "localhost", 8089);
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
