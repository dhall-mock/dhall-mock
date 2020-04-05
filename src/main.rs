use serde::Deserialize;

fn main() {
    println!("Hello from dhall mock project 👋");

    // Some Dhall data
    let data = r###"
        let Mock = ./dhall/Mock/package.dhall
        in Mock.HttpMethod.GET
    "###;

    // Deserialize it to a Rust type.
    let method: HttpMethod = serde_dhall::from_str(data).parse().unwrap();

    assert_eq!(method, HttpMethod::GET);
}

#[derive(Debug, Deserialize, PartialEq)]
enum HttpMethod {
    HEAD,
    GET,
    PUT,
    POST,
}
