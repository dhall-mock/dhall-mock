extern crate dhall_mock;

use std::panic;
use std::sync::{Arc, RwLock};
use tokio::runtime;
use tokio::sync::oneshot;

fn run_test<T>(test: T) -> ()
where
    T: FnOnce() -> () + panic::UnwindSafe,
{
    let mut web_rt = runtime::Runtime::new().unwrap();
    let (web_send_close, web_close_channel) = oneshot::channel::<()>();
    let (admin_send_close, admin_close_channel) = oneshot::channel::<()>();

    let join = setup(&web_rt, web_close_channel, admin_close_channel);

    let result = panic::catch_unwind(|| test());

    teardown(web_send_close, admin_send_close);

    let _ = web_rt.block_on(join);
    assert!(result.is_ok())
}

fn setup(
    web_rt: &runtime::Runtime,
    web_close_channel: oneshot::Receiver<()>,
    admin_close_channel: oneshot::Receiver<()>,
) -> tokio::task::JoinHandle<Result<(), anyhow::Error>> {
    let conf = dhall_mock::compiler::load_file("./dhall/static.dhall").unwrap();
    let expectations = dhall_mock::compiler::compile_configuration(conf.as_ref()).unwrap();

    let state = Arc::new(RwLock::new(dhall_mock::State {
        expectations: expectations,
    }));

    web_rt.spawn(dhall_mock::run_mock_server(
        String::from("0.0.0.0:8088"),
        String::from("0.0.0.0:8089"),
        state,
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
    run_test(|| {
        let api = format!("http://{}:{}/greet/pwet", "localhost", 8088);
        let req = reqwest::blocking::get(&api).unwrap();

        assert_eq!(reqwest::StatusCode::CREATED, req.status());
    })
}

#[test]
fn test_admin_api() {
    run_test(|| {
        let api = format!("http://{}:{}/expectations", "localhost", 8089);
        let req = reqwest::blocking::get(&api).unwrap();

        assert_eq!(reqwest::StatusCode::OK, req.status());
    })
}
