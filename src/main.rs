use anyhow::{Context, Error};
use serde::Deserialize;

fn main() -> Result<(), Error> {
    println!("Hello from dhall mock project 👋");

    // Some Dhall data
    let data = r###"
        let Mock = ./dhall/Mock/package.dhall
        in Mock.HttpMethod.UNKNOW
    "###;

    // Deserialize it to a Rust type.
    let method: HttpMethod = serde_dhall::from_str(data)
        .parse()
        .context("Parsing dhall configuration")?;

    assert_eq!(method, HttpMethod::GET);

    Ok(())
}

#[derive(Debug, Deserialize, PartialEq)]
enum HttpMethod {
    HEAD,
    GET,
    PUT,
    POST,
}
