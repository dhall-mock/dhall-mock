use std::sync::{Arc, RwLock};
use std::time::Instant;

use lazy_static::lazy_static;
use rayon::{ThreadPool, ThreadPoolBuilder};
use retry::retry;

use log::info;

use anyhow::{anyhow, Context, Error};

use super::compilation::compile_configuration;
use super::model::{Expectation, IncomingRequest};
use retry::delay::{jitter, Exponential};
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
pub async fn add_expectations_in_state(
    state: SharedState,
    mut expectations: Vec<Expectation>,
) -> Result<(), Error> {
    let mut state = retry(Exponential::from_millis(10).map(jitter).take(3), || {
        state.write()
    })
    .map_err(|_| anyhow!("Can't acquire write lock on state"))?;
    state.expectations.append(&mut expectations);
    Ok(())
}

pub async fn load_dhall_expectation(
    id: String,
    dhall_content: String,
) -> Result<Vec<Expectation>, Error> {
    let (s, r) = oneshot::channel();
    POOL.spawn(move || {
        info!("Start load {} config", id);
        let now = Instant::now();
        let result =
            compile_configuration(&dhall_content).context(format!("Error compiling {}", id));
        info!("Loaded {}, in {} secs", id, now.elapsed().as_secs());
        s.send(result)
            .expect("Internal error on communication between app and dhall runtimes");
    });
    r.await
        .expect("Internal error on communication between app and dhall runtimes")
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
