use std::env;

use anyhow::{anyhow, Error};
use env_logger::Env;
use log::info;

use crate::compiler::{compile_configuration, load_file};
use crate::expectation::model::HttpMethod;
use crate::mock::{look_for_expectation, IncomingRequest};

mod compiler;
mod expectation;
mod mock;

fn main() -> Result<(), Error> {
    start_logger();

    let args: Vec<String> = env::args().collect();
    let filename = args
        .get(1)
        .ok_or(anyhow!("Program need 1 argument : File configuration path"))?;

    info!("Hello from dhall mock project 👋");
    let configuration = load_file(filename)?;
    let expectations = compile_configuration(&configuration)?;

    let incoming_request = IncomingRequest {
        method: HttpMethod::GET,
        path: String::from("/greet/wololo"),
    };

    let selected_expectation = look_for_expectation(&expectations, incoming_request);

    info!("Loaded expectations : {:?}", expectations);
    info!("selected expectation : {:?}", selected_expectation);
    Ok(())
}

fn start_logger() {
    let env = Env::new().filter_or("RUST_LOG", "INFO");
    env_logger::init_from_env(env);
}
