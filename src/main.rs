pub mod client;
#[allow(unused_macros)]
pub mod config;
pub mod consts;
pub mod grpc_client;
pub mod grpc_models;
pub mod threads;
pub mod ui;
pub mod websocket_server;
pub mod socketio_client;
pub mod discord;
pub mod replay_recv;
#[cfg(target_os = "windows")] pub mod tray;

#[tokio::main]
async fn main() {
    use crate::config::cfg;
    use simple_logging::log_to_file;

    // Setup Logs
    if cfg().debug_logs && threads::port_is_available(18639) {
        log_to_file(config::get_log_file_path(), log::LevelFilter::Info).expect("Couldn't create logger!");
        log_panics::init();
    }

    threads::init().await;
}
