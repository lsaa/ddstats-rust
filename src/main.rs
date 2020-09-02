mod mem;
mod ui;
pub mod structs;
pub mod consts;
pub mod app;
pub mod utils;
pub mod timing;

use simple_logging::{log_to_file};
use std::time::Duration;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};

fn main() {
    let _ = log_to_file("ddstats-rust.log", log::LevelFilter::Info);
    log::info!("Initializing App...");

    let app = Arc::new(Mutex::new(app::App {
        state: structs::State::NotConnected, 
        data: None,
        game_pid: None,
        process_handle: None,
    }));

    //Game Capture - 36 times a second
    let cap = app.clone();
    std::thread::spawn(move || {
        loop {
            cap.lock().unwrap().tick();
            std::thread::sleep(Duration::from_secs_f32(1.0 / 36.0));
        }
    });

    //Socket and Ui- 3 times a second
    let sock = app.clone();
    loop {
        let pid = sock.lock().unwrap().game_pid;
        std::thread::sleep(Duration::from_secs_f32(1.0 / 3.0));
    }
}