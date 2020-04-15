extern crate dhall_mock;

use anyhow::{anyhow, Error};
use log::{debug, info, warn};
use signal_hook::iterator::Signals;

use std::sync::{Arc, RwLock};
use tokio::sync::mpsc::channel;

use dhall_mock::cli;
use dhall_mock::{compiler_executor, create_loader_runtime, run_web_server, start_logger, State};

fn main() -> Result<(), Error> {
    start_logger();

    let cli_args = cli::load_cli_args();

    let loading_rt = create_loader_runtime()?;
    let web_rt = tokio::runtime::Runtime::new()?;

    info!("Start dhall mock project 👋");
    let (mut tx, rx) = channel(10);
    let state = Arc::new(RwLock::new(State {
        expectations: vec![],
    }));

    // Start dhall configuration loader
    loading_rt.spawn(compiler_executor(rx, state.clone()));

    let http_bind = cli_args.http_bind.clone();

    // Send configuration to load from cli
    loading_rt.spawn(async move {
        for configuration in cli_args.configuration_files.iter() {
            debug!("Send configuration file {}", configuration);
            tx.send(configuration.clone()).await.unwrap_or_else(|err| {
                warn!(
                    "Error sending load trigger for {} : {:#}",
                    configuration, err
                )
            });
        }
    });

    // Start web server
    let (web_send_close, web_close_channel) = tokio::sync::oneshot::channel::<()>();
    web_rt.spawn(run_web_server(http_bind, state.clone(), web_close_channel));

    // Wait for signal
    let signals = Signals::new(&[signal_hook::SIGINT])?;
    match signals.forever().next() {
        Some(signal_hook::SIGINT) => {
            web_send_close
                .send(())
                .unwrap_or_else(|_| warn!("Error graceful shutdown"));
            Ok(())
        }
        _ => Err(anyhow!("Captured signal that should not be managed")),
    }
}
