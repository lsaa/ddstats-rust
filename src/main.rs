pub mod client;
pub mod config;
pub mod consts;
pub mod grpc_client;
pub mod grpc_models;
pub mod threads;
pub mod ui;
pub mod websocket_server;
pub mod socketio_client;
pub mod discord;
#[cfg(target_os = "windows")] pub mod tray;

#[tokio::main]
async fn main() {
    use crate::config::cfg;
    use simple_logging::log_to_file;

    // Setup Logs
    if cfg().debug_logs {
        log_to_file("debug_logs.txt", log::LevelFilter::Info).expect("Couldn't create logger!");
        log_panics::init();
    }

    threads::init().await;
}
