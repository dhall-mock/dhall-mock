use anyhow::{Context, Error};
use env_logger::Env;
use log::{debug, info, warn};

use crate::compiler::{compile_configuration, load_file};
use crate::expectation::model::{display_expectations, Expectation};
use std::sync::{Arc, RwLock};
use std::time::Instant;
use tokio::sync::mpsc::Receiver;
use tokio::task::spawn_blocking;

pub mod cli;
mod compiler;
mod expectation;
mod web;

pub struct State {
    pub expectations: Vec<Expectation>,
}

pub fn create_loader_runtime() -> Result<tokio::runtime::Runtime, Error> {
    tokio::runtime::Builder::new()
        .threaded_scheduler()
        .thread_stack_size(8 * 1024 * 1024)
        .core_threads(1)
        .max_threads(3)
        .build()
        .context("Error creating loader tokio runtime")
}

pub fn start_logger() {
    let env = Env::new().filter_or("RUST_LOG", "INFO");
    env_logger::init_from_env(env);
}

pub async fn run_web_server(
    http_bind: String,
    state: Arc<RwLock<State>>,
    close_channel: tokio::sync::oneshot::Receiver<()>,
) -> Result<(), Error> {
    web::web_server(state, http_bind, close_channel).await
}

pub async fn compiler_executor(mut receiver: Receiver<String>, state: Arc<RwLock<State>>) {
    debug!("Dhall compiler task started");
    while let Some(config) = receiver.recv().await {
        debug!("Received config to load from file {}", config);
        let state = state.clone();
        spawn_blocking(move || {
            info!("Start loading config {}", config);
            let now = Instant::now();
            let load_result = load_file(config.as_str())
                .and_then(|configuration| compile_configuration(&configuration))
                .context(format!("Error parsing dhall configuration {}", config));
            info!("Loaded {} in {} secs", config, now.elapsed().as_secs());
            match (load_result, state.write()) {
                (Ok(mut expectations), Ok(mut state)) => {
                    info!(
                        "Loaded expectations : {}",
                        display_expectations(&expectations)
                    );
                    state.expectations.append(expectations.as_mut());
                }
                (Err(e), _) => warn!("{:#}", e),
                (_, Err(e)) => warn!(
                    "Error mutating state for configuration {} , : {:#}",
                    config, e
                ),
            }
        });
    }
    receiver.close();
}
