use crate::expectation::model::Expectation;
use anyhow::{Context, Error};
use env_logger::Env;
use log::info;

mod expectation;

fn main() -> Result<(), Error> {
    start_logger();
    info!("Hello from dhall mock project 👋");

    // Some Dhall data
    let data = r###"
        let Mock = ./dhall/Mock/package.dhall
        in { request  = { method  = Some Mock.HttpMethod.GET
                         , path    = Some "/greet/pwet"
                         }
            , response = { statusCode   = Some +200
                         , statusReason = None Text
                         , body         = Some "Hello, pwet !"
                         }
            }
    "###;

    // Deserialize it to a Rust type.
    let method: Expectation = serde_dhall::from_str(data)
        .parse()
        .context("Error parsing shall configuration")?;
    info!("Loaded from dhall configuration : {:?}", method);

    Ok(())
}

fn start_logger() {
    let env = Env::new().filter_or("RUST_LOG", "INFO");
    env_logger::init_from_env(env);
}
