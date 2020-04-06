use std::env;

use anyhow::{anyhow, Error};
use env_logger::Env;
use log::info;

use crate::compiler::{compile_configuration, load_file};
use crate::expectation::model::Expectation;

mod expectation;
mod compiler;

fn main() -> Result<(), Error> {
    start_logger();

    let args: Vec<String> = env::args().collect();
    let filename = args.get(1).ok_or(anyhow!("Program need 1 argument : File configuration path"))?;

    info!("Hello from dhall mock project 👋");
    let configuration = load_file(filename)?;
    let expectations =  compile_configuration(&configuration)?;

    info!("Loaded expectations : {:?}", expectations);
    Ok(())

}

fn start_logger() {
    let env = Env::new().filter_or("RUST_LOG", "INFO");
    env_logger::init_from_env(env);
}
