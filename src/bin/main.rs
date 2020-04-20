extern crate dhall_mock;

use anyhow::{anyhow, Error};
use log::{info, warn};
use signal_hook::iterator::Signals;

use std::sync::{Arc, RwLock};
use std::time::Duration;

use dhall_mock::cli;
use dhall_mock::{
    create_loader_runtime, load_configuration_files, run_mock_server, start_logger, State,
};
use tokio::runtime::Runtime;
use tokio::sync::oneshot;

fn main() -> Result<(), Error> {
    start_logger();
    let mut web_rt = Runtime::new()?;
    let loading_rt = Arc::new(create_loader_runtime()?);

    let cli_args = cli::load_cli_args();

    info!("Start dhall mock project 👋");
    let state = Arc::new(RwLock::new(State {
        expectations: vec![],
    }));

    load_configuration_files(
        loading_rt.clone(),
        state.clone(),
        cli_args.configuration_files.into_iter()
    );

    // Start web server
    let (web_send_close, web_close_channel) = oneshot::channel::<()>();
    let (admin_send_close, admin_close_channel) = oneshot::channel::<()>();

    std::thread::spawn(move || {
        // Wait for signal
        let signals = Signals::new(&[signal_hook::SIGINT])?;
        match signals.forever().next() {
            Some(signal_hook::SIGINT) => {
                web_send_close
                    .send(())
                    .unwrap_or_else(|_| warn!("Error graceful shutdown"));
                admin_send_close
                    .send(())
                    .unwrap_or_else(|_| warn!("Error graceful shutdown"));
                Ok(())
            }
            _ => Err(anyhow!("Captured signal that should not be managed")),
        }
    });

    let result = web_rt.block_on(run_mock_server(
        cli_args.http_bind,
        cli_args.admin_http_bind,
        state.clone(),
        web_close_channel,
        admin_close_channel,
    ));
    web_rt.shutdown_timeout(Duration::from_secs(1));
    // Can't shutdown loading_rt as shutdown_timeout need to move value and we can't anymore since we are sharing via Arc this Runtime ...
    // loading_rt.shutdown_timeout(Duration::from_secs(1));
    result
}
