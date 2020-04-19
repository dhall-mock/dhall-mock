extern crate dhall_mock;

use anyhow::{anyhow, Error};
use log::{info, warn};
use signal_hook::iterator::Signals;

use std::sync::{Arc, RwLock};
use tokio::sync::mpsc::channel;

use dhall_mock::cli;
use dhall_mock::{
    compiler_executor, create_loader_runtime, load_configuration_files, run_web_server, load_configuration,
    start_logger, State,
};

fn main() -> Result<(), Error> {
    start_logger();

    let cli_args = cli::load_cli_args();

    let loading_rt = create_loader_runtime()?;
    let web_rt = tokio::runtime::Runtime::new()?;

    info!("Start dhall mock project 👋");
    let state = Arc::new(RwLock::new(State {
        expectations: vec![],
    }));

    let (config_sender, config_receiver) = channel(10);
    // Start dhall configuration loader
    loading_rt.spawn(compiler_executor(config_receiver, state.clone()));

    if cli_args.wait_configuration_load {
        // Load configuration files synchronously
        info!("Wait for configuration loading");
        for config in cli_args.configuration_files {
            load_configuration(config, state.clone())
        }
    } else {
        // Load configuration files asynchronously
        loading_rt.spawn(load_configuration_files(
            cli_args.configuration_files.into_iter(),
            config_sender,
        ));
    }

    // Start web server
    let (web_send_close, web_close_channel) = tokio::sync::oneshot::channel::<()>();
    web_rt.spawn(run_web_server(
        cli_args.http_bind,
        state.clone(),
        web_close_channel,
    ));

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
