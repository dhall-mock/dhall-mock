extern crate dhall_mock;

use std::sync::{Arc, RwLock};
use tokio::runtime;
use tokio::sync::oneshot;

fn setup(web_rt: &runtime::Runtime, web_close_channel: oneshot::Receiver<()>) {
    let conf = dhall_mock::compiler::load_file("./dhall/static.dhall").unwrap();
    let expectations = dhall_mock::compiler::compile_configuration(conf.as_ref()).unwrap();

    let state = Arc::new(RwLock::new(dhall_mock::State {
        expectations: expectations,
    }));

    web_rt.spawn(dhall_mock::run_web_server(
        String::from("0.0.0.0:8088"),
        state,
        web_close_channel,
    ));
}

fn stop(web_send_close: oneshot::Sender<()>) {
    web_send_close.send(()).unwrap()
}

#[test]
fn test_api() {
    let web_rt = runtime::Runtime::new().unwrap();
    let (web_send_close, web_close_channel) = oneshot::channel::<()>();
    setup(&web_rt, web_close_channel);

    let api = format!("http://{}:{}/greet/pwet", "localhost", 8088);
    let req = reqwest::blocking::get(&api).unwrap();

    assert_eq!(reqwest::StatusCode::CREATED, req.status());

    stop(web_send_close);
}
