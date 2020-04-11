use anyhow::Error;
use env_logger::Env;
use log::{debug, info};

use crate::compiler::ConfLoadingFuture;
use crate::expectation::model::Expectation;
use crate::web::State;
use std::sync::{Arc, RwLock};

mod cli;
mod compiler;
mod expectation;
mod mock;
mod web;

#[tokio::main]
async fn main() -> Result<(), Error> {
    start_logger();

    let cli_args = cli::load_cli_args();

    info!("Start dhall mock project 👋");

    let mut expectations: Vec<Expectation> = vec![];
    for configuration_file in cli_args.configuration_files {
        debug!("Loading configuration file {}", configuration_file);

        let res = &mut ConfLoadingFuture::load_file(&configuration_file).await;
        expectations.append(res);
    }
    info!("Loaded expectations : {:?}", expectations);

    let state = Arc::new(RwLock::new(State { expectations }));
    web::web_server(state.clone(), cli_args.http_bind).await?;

    Ok(())
}

fn start_logger() {
    let env = Env::new().filter_or("RUST_LOG", "INFO");
    env_logger::init_from_env(env);
}
