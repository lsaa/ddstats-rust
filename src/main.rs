pub mod client;
#[allow(unused_macros)] pub mod config;
pub mod consts;
pub mod grpc_client;
#[allow(unused_macros)] pub mod grpc_models;
pub mod threads;
pub mod ui;
pub mod websocket_server;
pub mod socketio_client;
pub mod discord;
pub mod replay_recv;
pub mod updater;
#[cfg(target_os = "windows")] pub mod tray;

#[tokio::main]
async fn main() {
    use crate::config::cfg;
    use simple_logging::log_to_file;

    if cfg().auto_updater && !cfg().offline {
        tokio::task::spawn_blocking(|| {
            if let Ok(()) = updater::update() {
                let current_exe = std::env::current_exe().expect("current exe");
                restart_process(current_exe);
            }
        }).await.unwrap();
    }

    // Setup Logs
    if cfg().debug_logs && threads::port_is_available(18639) {
        log_to_file(config::get_log_file_path(), log::LevelFilter::Info).expect("Couldn't create logger!");
        log_panics::init();
    }

    threads::init().await;
}

#[cfg(unix)]
fn restart_process(current_exe: std::path::PathBuf) {
    use std::os::unix::process::CommandExt as _;
    std::thread::sleep(std::time::Duration::from_secs(5));
    let err = std::process::Command::new(current_exe)
        .args(std::env::args().into_iter().skip(1))
        .exec();
    panic!("Failed to restart: {}", err);
}

#[cfg(windows)]
fn restart_process(current_exe: std::path::PathBuf) {
    std::thread::sleep(std::time::Duration::from_secs(5));
    std::process::Command::new(current_exe)
        .args(std::env::args().into_iter().skip(1))
        .spawn().expect("restarted");
}
