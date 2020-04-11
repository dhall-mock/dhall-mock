use anyhow::{Context, Error};
use env_logger::Env;
use log::{debug, info, warn};

use crate::compiler::{compile_configuration, load_file, ConfLoadingFuture};
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

    let expectations = cli_args
        .configuration_files
        .iter()
        .flat_map(|configuration_file| {
            debug!("Loading configuration file {}", configuration_file);

            // TODO how can I run my future?
            let configuration_result = ConfLoadingFuture.load_file(configuration_file);

            match configuration_result {
                Ok(expectations) => expectations.into_iter(),
                Err(e) => {
                    warn!("{}", e);
                    Vec::new().into_iter()
                }
            }
        })
        .collect();
    info!("Loaded expectations : {:?}", expectations);

    let state = Arc::new(RwLock::new(State { expectations }));
    web::web_server(state.clone(), cli_args.http_bind).await?;

    Ok(())
}

fn start_logger() {
    let env = Env::new().filter_or("RUST_LOG", "INFO");
    env_logger::init_from_env(env);
}
