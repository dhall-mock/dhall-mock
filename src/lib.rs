use anyhow::{anyhow, Context, Error};
use env_logger::Env;

use web::admin::{server as admin_server, AdminServerContext};
use web::mock::{server as mock_server, MockServerContext};

pub mod mock;
pub mod web;

pub fn start_logger() -> Result<(), Error> {
    let env = Env::new().filter_or("RUST_LOG", "INFO");
    env_logger::try_init_from_env(env)
        .map_err(|e| anyhow!(e))
        .context("Error creating logger.")
}

pub async fn start_servers(
    mock_context: MockServerContext,
    admin_context: AdminServerContext,
) -> Result<(), Error> {
    tokio::try_join!(mock_server(mock_context), admin_server(admin_context),)
        .map(|_| ())
        .context("Error on running web servers")
}
