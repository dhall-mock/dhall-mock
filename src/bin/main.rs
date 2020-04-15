use anyhow::Error;
use env_logger::Env;

extern crate dhall_mock;

fn main() -> Result<(), Error> {
    start_logger();

    let cli_args = dhall_mock::cli::load_cli_args();

    let mut web_rt = tokio::runtime::Runtime::new()?;

    web_rt.block_on(async {
        let (_, rx_server) = tokio::sync::oneshot::channel();
        dhall_mock::run(cli_args, rx_server).await.unwrap();
        Ok(())
    })
}

fn start_logger() {
    let env = Env::new().filter_or("RUST_LOG", "INFO");
    env_logger::init_from_env(env);
}
