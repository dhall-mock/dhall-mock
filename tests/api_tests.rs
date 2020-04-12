extern crate dhall_mock;

struct TestConf {
    host: String,
    port: i32,
}

fn load_conf() -> TestConf {
    TestConf {
        host: String::from("localhost"),
        port: 8088,
    }
}

fn setup() {
    dhall_mock::run().unwrap()
}

#[test]
fn test_api() {
    setup();

    let test_conf = load_conf();

    let api = format!("http://{}:{}/greet/pwet", test_conf.host, test_conf.port);
    let req = reqwest::blocking::get(&api).unwrap();

    assert_eq!(reqwest::StatusCode::CREATED, req.status());
}
