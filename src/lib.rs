use anyhow::{Context, Error};
use log::{debug, info, warn};

use crate::compiler::{compile_configuration, load_file};
use crate::expectation::model::display_expectations;
use crate::web::State;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use tokio::sync::mpsc::{channel, Receiver};
use tokio::sync::oneshot;
use tokio::task::spawn_blocking;
use tokio::task::JoinHandle;

pub mod cli;
mod compiler;
mod expectation;
mod web;

pub async fn run(cli_args: cli::CliOpt) -> Result<(oneshot::Sender<()>, JoinHandle<()>), Error> {
    let loading_rt = tokio::runtime::Builder::new()
        .threaded_scheduler()
        .thread_stack_size(8 * 1024 * 1024)
        .core_threads(1)
        .max_threads(3)
        .build()?;

    info!("Start dhall mock project 👋");
    let (mut tx, rx) = channel(10);
    let state = Arc::new(RwLock::new(State {
        expectations: vec![],
    }));

    let compiler_state = state.clone();
    loading_rt.spawn(compiler_executor(rx, compiler_state));

    for configuration in cli_args.configuration_files.iter() {
        debug!("Send configuration file {}", configuration);
        tx.send(configuration.clone()).await?;
    }

    let (tx_server, rx_server) = tokio::sync::oneshot::channel();

    let join =
        tokio::task::spawn(
            async move { web::web_server(state, cli_args.http_bind, rx_server).await },
        );

    Ok((tx_server, join))
}

async fn compiler_executor(mut receiver: Receiver<String>, state: Arc<RwLock<State>>) {
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
