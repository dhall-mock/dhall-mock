extern crate dhall_mock;

use anyhow::Error;
use log::{debug, info};

use std::sync::{Arc, RwLock};
use tokio::sync::mpsc::channel;

use dhall_mock::cli;
use dhall_mock::{State, compiler_executor, create_loader_runtime, run_web_server, start_logger};

fn main() -> Result<(), Error> {
    start_logger();

    let cli_args = cli::load_cli_args();

    let loading_rt = create_loader_runtime()?;

    let mut web_rt = tokio::runtime::Runtime::new()?;

    info!("Start dhall mock project 👋");
    let (mut tx, rx) = channel(10);
    let state = Arc::new(RwLock::new(State {
        expectations: vec![],
    }));

    loading_rt.spawn(compiler_executor(rx, state.clone()));

    web_rt.block_on(async move {
        for configuration in cli_args.configuration_files.iter() {
            debug!("Send configuration file {}", configuration);
            tx.send(configuration.clone()).await?;
        }
        run_web_server(cli_args.http_bind, state.clone()).await
    })
}
