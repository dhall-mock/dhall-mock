use env_logger::Env;

use std::future::Future;

use anyhow::{anyhow, Context, Error};

use web::admin::{server as admin_server, AdminServerContext};
use web::mock::{server as mock_server, MockServerContext};

pub mod mock;
pub mod web;

pub fn create_loader_runtime() -> Result<tokio::runtime::Runtime, Error> {
    tokio::runtime::Builder::new()
        .threaded_scheduler()
        .thread_stack_size(8 * 1024 * 1024)
        .core_threads(1)
        .max_threads(3)
        .build()
        .context("Error creating loader tokio runtime")
}

pub fn start_logger() -> Result<(), Error> {
    let env = Env::new().filter_or("RUST_LOG", "INFO");
    env_logger::try_init_from_env(env)
        .map_err(|e| anyhow!(e))
        .context("Error creating logger.")
}

pub async fn start_servers(
    mock_context: MockServerContext,
    admin_context: AdminServerContext,
    // sigint: impl Future<Output = Result<(), Error>>,
) -> Result<(), Error> {
    tokio::try_join!(mock_server(mock_context), admin_server(admin_context),)
        .map(|_| ())
        .context("Error on running web servers")
}
