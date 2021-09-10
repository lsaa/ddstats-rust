pub mod config;
pub mod consts;
pub mod grpc_models;
#[macro_use]
pub mod utils;
pub mod client;
pub mod mem;
pub mod ui;

pub mod threads;

use std::sync::{mpsc, Arc, RwLock};

use mem::StatsBlockWithFrames;
use simple_logging::log_to_file;
use threads::{GameClientThread, UiThread};

fn main() {
    let cfg = config::CONFIG.with(|z| z.clone());
    if cfg.debug_logs {
        log_to_file("debug_logs.txt", log::LevelFilter::Info).expect("Couldn't create logger!");
    }

    let last_poll: Arc<RwLock<StatsBlockWithFrames>> = Arc::new(RwLock::default());
    let logs: Arc<RwLock<Vec<String>>> = Arc::new(RwLock::default());
    let (submit_event_sender, submit_event_receiver) = mpsc::channel();
    let (log_sender, log_recevicer) = mpsc::channel::<String>();
    let (game_connected_sender, game_connected_receiver) = mpsc::channel::<bool>();
    let (game_disconnected_sender, game_disconnected_receiver) = mpsc::channel::<bool>();
    let game_connected = Arc::new(RwLock::new(Conn { is_ok: false }));

    let _game_thread = GameClientThread::create_and_start(
        last_poll.clone(),
        submit_event_sender,
        log_sender,
        game_disconnected_sender,
        game_connected_sender,
    );

    let _ui_thread =
       UiThread::create_and_start(last_poll.clone(), logs.clone(), game_connected.clone());

    loop {
        if let Ok(new_log) = log_recevicer.try_recv() {
            if let Ok(mut writer) = logs.try_write() {
                writer.push(new_log);
            }
        }

        if let Ok(_) = game_disconnected_receiver.try_recv() {
            if let Ok(mut writer) = game_connected.try_write() {
                writer.is_ok = false;
            }
        }

        if let Ok(_) = game_connected_receiver.try_recv() {
            if let Ok(mut writer) = game_connected.try_write() {
                writer.is_ok = true;
            }
        }

        if let Ok(_game) = submit_event_receiver.try_recv() {}
    }
}

pub struct Conn {
    pub is_ok: bool,
}
