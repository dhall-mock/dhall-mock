extern crate dhall_mock;

use std::fs;
use std::sync::{Arc, RwLock};

use anyhow::{Context, Error};
use futures::future::join_all;
use log::{info, warn};
use structopt::StructOpt;

use dhall_mock::mock::service::{
    add_expectations_in_state, load_dhall_expectation, SharedState, State,
};
use dhall_mock::web::admin::AdminServerContext;
use dhall_mock::web::mock::MockServerContext;
use dhall_mock::{start_logger, start_servers};
use futures::TryFutureExt;

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

#[tokio::main]
async fn main() -> Result<(), Error> {
    start_logger()?;

    let cli_args = CliOpt::from_args();

    info!("Start dhall mock project 👋");
    let state = Arc::new(RwLock::new(State {
        expectations: vec![],
    }));

    let load_configurations = join_all(
        cli_args
            .configuration_files
            .into_iter()
            .map(|configuration| load_configuration_file(state.clone(), configuration)),
    );

    if cli_args.wait {
        load_configurations.await;
    } else {
        tokio::task::spawn(load_configurations);
    }

    let mock_server_context = MockServerContext {
        http_bind: cli_args.http_bind,
        state: state.clone(),
    };

    let admin_server_context = AdminServerContext {
        http_bind: cli_args.admin_http_bind,
        state,
    };

    start_servers(mock_server_context, admin_server_context).await
}

async fn load_configuration_file(
    state: SharedState,
    configuration_name: String,
) -> Result<(), Error> {
    let configuration = fs::read_to_string(configuration_name.as_str())
        .context(format!("Error reading file {} content", configuration_name))?;
    match load_dhall_expectation(configuration_name.clone(), configuration)
        .and_then(|expectations| add_expectations_in_state(state, expectations))
        .await
    {
        Ok(()) => info!("Configuration {} loaded", configuration_name),
        Err(e) => warn!(
            "Error loading configuration {} : {:#}",
            configuration_name, e
        ),
    };
    Ok(())
}
