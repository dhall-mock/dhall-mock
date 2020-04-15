use anyhow::Error;
use env_logger::Env;

extern crate dhall_mock;

fn main() -> Result<(), Error> {
    start_logger();

    let cli_args = dhall_mock::cli::load_cli_args();

    let _ = dhall_mock::run(cli_args);
    Ok(())
}

fn start_logger() {
    let env = Env::new().filter_or("RUST_LOG", "INFO");
    env_logger::init_from_env(env);
}
