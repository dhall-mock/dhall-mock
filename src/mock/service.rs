use std::sync::{Arc, RwLock};
use std::time::Instant;

use log::info;
use tokio::task::block_in_place;

use anyhow::{anyhow, Context, Error};

use super::compilation::compile_configuration;
use super::model::{Expectation, IncomingRequest};

pub struct State {
    pub expectations: Vec<Expectation>,
}

pub type SharedState = Arc<RwLock<State>>;

// TODO add unit tests
pub async fn add_configuration(
    state: SharedState,
    id: String,
    configuration: String,
) -> Result<(), Error> {
    info!("Start load {} config", id);
    let now = Instant::now();
    let result = block_in_place(move || compile_configuration(&configuration))
        .context(format!("Error compiling {}", id));
    info!("Loaded {}, in {} secs", id, now.elapsed().as_secs());
    let mut expectation = result?;
    let mut state = state
        .write()
        .map_err(|_| anyhow!("Can't acquire write lock on state"))?;
    state.expectations.append(&mut expectation);
    Ok(())
}

// Todo add Unit tests
pub async fn search_for_mock(
    request: IncomingRequest,
    state: SharedState,
) -> Result<Option<Expectation>, Error> {
    let state = state
        .read()
        .map_err(|_| anyhow!("Error acquiring read on shared state"))?;

    Ok(Expectation::look_for_expectation(&state.expectations, &request).cloned())
}
