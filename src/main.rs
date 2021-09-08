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
use threads::{GameClientThread, UiThread};

fn main() {
    let last_poll: Arc<RwLock<StatsBlockWithFrames>> = Arc::new(RwLock::default());
    let logs: Arc<RwLock<Vec<String>>> = Arc::new(RwLock::default());
    let (submit_event_sender, submit_event_receiver) = mpsc::channel();
    let (log_sender, log_recevicer) = mpsc::channel::<String>();

    let _game_thread =
        GameClientThread::create_and_start(last_poll.clone(), submit_event_sender, log_sender);

    let _ui_thread = UiThread::create_and_start(last_poll.clone(), logs.clone());

    loop {
        if let Ok(new_log) = log_recevicer.try_recv() {
            if let Ok(mut writer) = logs.try_write() {
                writer.push(new_log);
            }
        }

        if let Ok(_game) = submit_event_receiver.try_recv() {}
    }
}
