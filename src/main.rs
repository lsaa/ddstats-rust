pub mod app;
pub mod config;
pub mod consts;
mod mem;
pub mod net;
pub mod socket;
pub mod structs;
pub mod timing;
pub mod ui;
pub mod utils;

use crate::structs::AppDataExtraction;
use simple_logging::log_to_file;
use std::sync::{Arc, Mutex};
use std::{time::Duration};

fn main() {
    let _ = log_to_file("ddstats-rust.log", log::LevelFilter::Info);
    log::info!("Initializing App...");
    let _ = net::get_motd();
    let mut old_ui = false;
    for argument in std::env::args() {
        if argument == "--old-ui" {
            old_ui = true;
        }
    }
    let mut scheduler = timing::Scheduler::new();
    if !old_ui { ui::setup(); }

    let app = Arc::new(Mutex::new(app::App {
        state: structs::State::NotConnected,
        data: None,
        game_pid: None,
        process_handle: None,
        data_members: None,
        survival_file_path: String::new(),
        can_submit_run: true,
        old_ui,
        logs: vec![],
    }));

    let cap = app.clone();
    scheduler.create_task(timing::TemporalTask::new(
        Duration::from_secs_f32(1.0 / 36.0),
        capture_game,
        cap,
    ));
    let cap = app.clone();
    scheduler.create_task(timing::TemporalTask::new(
        Duration::from_secs_f32(1.0 / 3.0),
        socket_ui,
        cap
    ));

    loop {
        scheduler.execute_pending();
        std::thread::sleep(Duration::from_nanos(50000));
    }
}

type AMA = Arc<Mutex<app::App>>;

//Game Capture - 36 times a second
fn capture_game(args: AMA) {
    std::thread::spawn(move || {
        let lock = args.try_lock();
        if lock.is_ok() {
            lock.unwrap().tick();
        }
    });
}

//Socket and Ui- 3 times a second
fn socket_ui(args: AMA) {
    std::thread::spawn(move || {
        let lock = args.try_lock();
        if lock.is_ok() {
            let data = lock.as_ref().unwrap();
            let extraction = AppDataExtraction::from_app(data);
            drop(data); // free App for the other threads
            unsafe { if ui::TERMINAL.is_some() {
                let _ = ui::draw(&extraction);
            }}
        }
    });
}