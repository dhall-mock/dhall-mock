use std::sync::{Arc, RwLock};
use std::time::Instant;

use lazy_static::lazy_static;
use rayon::{ThreadPool, ThreadPoolBuilder};

use log::info;

use anyhow::{anyhow, Context, Error};

use super::compilation::compile_configuration;
use super::model::{Expectation, IncomingRequest};
use tokio::sync::oneshot;

pub struct State {
    pub expectations: Vec<Expectation>,
}

pub type SharedState = Arc<RwLock<State>>;

lazy_static! {
    static ref POOL: ThreadPool = ThreadPoolBuilder::new()
        .num_threads(3)
        .stack_size(8 * 1024 * 1024)
        .build()
        .unwrap();
}

// Todo add Unit tests
pub async fn load_configuration(
    state: SharedState,
    id: String,
    configuration: String,
) -> Result<(), Error> {
    let (s, r) = oneshot::channel();
    POOL.spawn(move || {
        info!("Start load {} config", id);
        let now = Instant::now();
        let result =
            compile_configuration(&configuration).context(format!("Error compiling {}", id));
        info!("Loaded {}, in {} secs", id, now.elapsed().as_secs());
        s.send(result).expect("Error sending result into channel");
    });
    // TODO change unwrap
    let result = r.await.expect("Error listening result into channel");
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
