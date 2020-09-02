mod mem;
mod ui;
pub mod structs;
pub mod consts;
pub mod app;
pub mod utils;
pub mod timing;

use simple_logging::{log_to_file};
use std::time::Duration;
use std::sync::{Arc, Mutex};

fn main() {
    let _ = log_to_file("ddstats-rust.log", log::LevelFilter::Info);
    log::info!("Initializing App...");

    let mut scheduler = timing::Scheduler::new();

    let app = Arc::new(Mutex::new(app::App {
        state: structs::State::NotConnected, 
        data: None,
        game_pid: None,
        process_handle: None,
    }));

    //Game Capture - 36 times a second
    fn capture_game(game_capture_mutex: AMA) {
        std::thread::spawn(move || {
            let lock = game_capture_mutex.try_lock();
            if lock.is_ok() {
                lock.unwrap().tick();
            }
        });
    }

    //Socket and Ui- 3 times a second
    fn socket_ui(game_capture_mutex: AMA) {
        std::thread::spawn(move || {
            let lock= game_capture_mutex.try_lock();
            if lock.is_ok() {
                //println!("{:?}", lock.unwrap().data.as_ref().unwrap().last_fetch_data.as_ref().unwrap().timer);
            }
        });
    }

    let cap = app.clone();
    scheduler.create_task(timing::TemporalTask::new(Duration::from_secs_f32(1.0 / 36.0), capture_game, cap));
    let cap = app.clone();
    scheduler.create_task(timing::TemporalTask::new(Duration::from_secs_f32(1.0 / 3.0), socket_ui, cap));

    loop {
        scheduler.execute_pending();
    }
}

type AMA = Arc<Mutex<app::App>>;