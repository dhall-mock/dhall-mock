use anyhow::{Context, Error};
use crate::expectation::model::Expectation;

mod expectation;

fn main() -> Result<(), Error> {
    println!("Hello from dhall mock project 👋");

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
    let method: Expectation = serde_dhall::from_str(data).parse()?;
    println!("Loaded from dhall configuration : {:?}", method);
    Ok(())
}
