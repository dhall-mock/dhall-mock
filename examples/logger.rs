use env_logger::Env;
use log::{debug, error, info, trace, warn};

// Default level = ERROR

fn main() {
    // Set env to define RUST_LOG if not defined to INFO
    let env = Env::new().filter_or("RUST_LOG", "INFO");
    env_logger::init_from_env(env);

    error!("Error log");
    warn!("Warn log");
    info!("Info log");
    debug!("Debug trace");
    trace!("Trace log");
}
