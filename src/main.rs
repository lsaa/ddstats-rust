pub mod client;
pub mod config;
pub mod consts;
pub mod grpc_client;
pub mod grpc_models;
pub mod mem;
pub mod threads;
pub mod ui;
pub mod web_clients;
pub mod websocket_server;
pub mod socketio_client;
pub mod crypto_encoder;

#[tokio::main]
async fn main() {
    use simple_logging::log_to_file;
    let cfg = config::cfg();

    // Setup Logs
    if cfg.debug_logs {
        log_to_file("debug_logs.txt", log::LevelFilter::Info).expect("Couldn't create logger!");
        log_panics::init();
    }

    threads::MainTask::init().await;
}
