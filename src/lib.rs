use std::sync::{Arc, RwLock};
use std::time::Instant;

use env_logger::Env;
use log::{info, warn};
use tokio::runtime::Runtime;
use tokio::task::block_in_place;

use anyhow::{anyhow, Context, Error};

use crate::compiler::{compile_configuration, load_file};
use crate::expectation::model::Expectation;

pub mod cli;
pub mod compiler;
pub mod expectation;
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

pub async fn run_mock_server(
    http_bind: String,
    admin_http_bind: String,
    state: Arc<RwLock<State>>,
    loader_rt: Arc<Runtime>,
    close_web_channel: tokio::sync::oneshot::Receiver<()>,
    close_admin_channel: tokio::sync::oneshot::Receiver<()>,
) -> Result<(), Error> {
    let server = run_web_server(http_bind, state.clone(), close_web_channel);
    let admin_server = run_admin_server(
        admin_http_bind,
        state.clone(),
        loader_rt,
        close_admin_channel,
    );

    tokio::try_join!(server, admin_server)
        .map(|_| ())
        .context("Error on running web servers")
}

pub async fn run_web_server(
    http_bind: String,
    state: Arc<RwLock<State>>,
    close_channel: tokio::sync::oneshot::Receiver<()>,
) -> Result<(), Error> {
    web::web_server(state, http_bind, close_channel).await
}

pub async fn run_admin_server(
    http_bind: String,
    state: Arc<RwLock<State>>,
    loader_rt: Arc<Runtime>,
    close_channel: tokio::sync::oneshot::Receiver<()>,
) -> Result<(), Error> {
    web::admin_server(state, http_bind, loader_rt, close_channel).await
}

pub fn load_configuration_files(
    target_runtime: Arc<Runtime>,
    state: Arc<RwLock<State>>,
    configurations: impl Iterator<Item = String>,
) {
    for configuration in configurations {
        // TODO manage error
        let configuration_content = load_file(configuration.as_str()).unwrap();
        let state = state.clone();
        target_runtime.spawn(async move {
            match load_configuration(state, configuration.clone(), configuration_content).await {
                Ok(()) => info!("Configuration {} loaded", configuration),
                Err(e) => warn!("Error loading configuration {} : {:#}", configuration, e),
            }
        });
    }
}

pub async fn load_configuration(
    state: Arc<RwLock<State>>,
    id: String,
    configuration: String,
) -> Result<(), Error> {
    info!("Start load {} config", id);
    let now = Instant::now();
    let result = block_in_place(move || compile_configuration(&configuration))
        .context(format!("Error compiling {}", id));
    info!("Loaded {}, in {} secs", id, now.elapsed().as_secs());
    let mut expectation = result?;
    let mut state = state
        .write()
        .map_err(|_| anyhow!("Can't acquire write lock on state"))?;
    state.expectations.append(&mut expectation);
    Ok(())
}
