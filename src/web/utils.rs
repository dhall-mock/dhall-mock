use log::info;
use tokio::signal;

pub async fn sigint(service: String) {
    signal::ctrl_c().await.expect("failed to listen for event");
    info!("shutdown : {}", service);
}
