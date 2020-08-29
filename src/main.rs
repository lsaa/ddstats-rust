mod mem;
mod ui;
pub mod structs;
pub mod consts;
pub mod app;
pub mod utils;

use clokwerk::Scheduler;
use std::{thread, time};

fn main() {
    println!("Waiting for game...");

    let mut app = app::App {
        state: structs::State::NotConnected, 
        previous_data: None,
        game_pid: None,
        process_handle: None,
        scheduler: Scheduler::new(),
    };

    loop {
        app.tick();
        thread::sleep(time::Duration::from_millis(28)); // TODO - Actual timing mechanisms lol
    }
}
