use env_logger::Env;

use anyhow::{Context, Error};

use crate::mock::model::Expectation;
use web::admin::{server as admin_server, AdminServerContext};
use web::mock::{server as mock_server, MockServerContext};

pub mod mock;
pub mod web;

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

pub async fn start_servers(
    mock_context: MockServerContext,
    admin_context: AdminServerContext,
) -> Result<(), Error> {
    tokio::try_join!(mock_server(mock_context), admin_server(admin_context))
        .map(|_| ())
        .context("Error on running web servers")
}
