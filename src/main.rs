use std::env;

use anyhow::{anyhow, Error};
use env_logger::Env;
use log::info;

use crate::compiler::{compile_configuration, load_file};
use crate::web::State;
use std::sync::{Arc, RwLock};

mod compiler;
mod expectation;
mod mock;
mod web;

#[tokio::main]
async fn main() -> Result<(), Error> {
    start_logger();

    let args: Vec<String> = env::args().collect();
    let filename = args
        .get(1)
        .ok_or(anyhow!("Program need 1 argument : File configuration path"))?;

    info!("Start dhall mock project 👋");
    let configuration = load_file(filename)?;
    let expectations = compile_configuration(&configuration)?;
    info!("Loaded expectations : {:?}", expectations);

    let state = Arc::new(RwLock::new(State { expectations }));
    // TODO load http bind from config
    web::web_server(state.clone(), "0.0.0.0:8088".to_string()).await?;

    Ok(())
}

fn start_logger() {
    let env = Env::new().filter_or("RUST_LOG", "INFO");
    env_logger::init_from_env(env);
}
