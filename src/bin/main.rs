extern crate dhall_mock;

use anyhow::{anyhow, Context, Error};
use log::{info, warn};

use std::sync::{Arc, RwLock};
use std::time::Duration;

use dhall_mock::mock::service::{add_configuration, SharedState, State};
use dhall_mock::web::admin::AdminServerContext;
use dhall_mock::web::mock::MockServerContext;
use dhall_mock::{create_loader_runtime, start_logger, start_servers};
use std::borrow::BorrowMut;
use std::fs;
use structopt::StructOpt;
use tokio::runtime::Runtime;
use tokio::signal;
use tokio::sync::oneshot;

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "dhall-mock")]
struct CliOpt {
    /// Dhall configuration files to parse
    configuration_files: Vec<String>,
    /// http binding for server
    #[structopt(short, long, default_value = "0.0.0.0:8088")]
    http_bind: String,
    /// http binding for admin server
    #[structopt(short, long, default_value = "0.0.0.0:8089")]
    admin_http_bind: String,
    /// wait to compile all configuration files before starting web servers
    #[structopt(short, long)]
    wait: bool,
}

fn main() -> Result<(), Error> {
    start_logger()?;
    let mut web_rt = Runtime::new()?;
    let mut loading_rt = create_loader_runtime()?;

    let cli_args = CliOpt::from_args();

    info!("Start dhall mock project 👋");
    let state = Arc::new(RwLock::new(State {
        expectations: vec![],
    }));

    load_configuration_files(
        loading_rt.borrow_mut(),
        cli_args.wait,
        state.clone(),
        cli_args.configuration_files.into_iter(),
    );

    // Start web server
    let (web_send_close, web_close_channel) = oneshot::channel::<()>();
    let (admin_send_close, admin_close_channel) = oneshot::channel::<()>();

    //signal handler
    web_rt.spawn(async move {
        sigint_handling(web_send_close, admin_send_close).await;
    });

    let mock_server_context = MockServerContext {
        http_bind: cli_args.http_bind,
        state: state.clone(),
        close_channel: web_close_channel,
    };

    let admin_server_context = AdminServerContext {
        http_bind: cli_args.admin_http_bind,
        state: state.clone(),
        close_channel: admin_close_channel,
        target_runtime: Arc::new(loading_rt),
    };

    let result = web_rt.block_on(start_servers(mock_server_context, admin_server_context));

    web_rt.shutdown_timeout(Duration::from_secs(10));
    // Can't shutdown loading_rt as shutdown_timeout need to move value and we can't anymore since we are sharing via Arc this Runtime ...
    // loading_rt.shutdown_timeout(Duration::from_secs(1));
    result
}

fn load_configuration_files(
    target_runtime: &mut Runtime,
    wait: bool,
    state: SharedState,
    configurations: impl Iterator<Item = String>,
) {
    for configuration in configurations {
        match fs::read_to_string(configuration.as_str())
            .context(format!("Error reading file {} content", configuration))
        {
            Ok(configuration_content) => {
                let state = state.clone();
                let load = move || match add_configuration(
                    state,
                    configuration.clone(),
                    configuration_content,
                ) {
                    Ok(()) => info!("Configuration {} loaded", configuration),
                    Err(e) => warn!("Error loading configuration {} : {:#}", configuration, e),
                };
                if wait {
                    load();
                } else {
                    target_runtime.spawn(async move { tokio::task::block_in_place(load) });
                }
            }
            Err(e) => warn!("Error loading configuration file : \n{:#}", e),
        }
    }
}

async fn sigint_handling(
    web_send_close: oneshot::Sender<()>,
    admin_send_close: oneshot::Sender<()>,
) -> Result<(), Error> {
    signal::ctrl_c().await.expect("failed to listen for event");

    println!("ctrl-c received!");
    web_send_close
        .send(())
        .unwrap_or_else(|_| warn!("Error graceful shutdown"));
    admin_send_close
        .send(())
        .unwrap_or_else(|_| warn!("Error graceful shutdown"));

    Ok(())
}
