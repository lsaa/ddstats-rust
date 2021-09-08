//
// Thread Configs
//

use tui::layout::{Constraint, Direction, Layout};

use crate::{Conn, client::{Client, GameClientState, SubmitGameEvent}, config, mem::{GameConnection, StatsBlockWithFrames}, ui::draw_levi};
use std::{ops::BitAnd, sync::{mpsc::Sender, Arc, RwLock}, thread::{self, JoinHandle}, time::{Duration, Instant}};

/* Game Poll Thread */
pub struct GameClientThread {
    pub join_handle: JoinHandle<()>,
}

impl GameClientThread {
    pub fn create_and_start(
        last_poll: ArcRw<StatsBlockWithFrames>,
        sender: Sender<SubmitGameEvent>,
        log_sender: Sender<String>,
        game_disconnected: Sender<bool>,
        game_conneceted: Sender<bool>
    ) -> Self {
        let mut client = Client {
            game_connection: GameConnection::dead_connection(),
            game_state: GameClientState::NotConnected,
            last_game_update: Instant::now(),
            compiled_run: None,
            log_sender: log_sender.clone(),
            conn: (game_conneceted, game_disconnected)
        };

        let join_handle = thread::spawn(move || loop {
            client.game_loop();

            if let Some(data) = &client.game_connection.last_fetch {
                if let Ok(mut writer) = last_poll.write() {
                    writer.clone_from(data);
                }
            }

            if let Some(run_to_submit) = &client.compiled_run {
                if !run_to_submit.1 {
                    log_sender.send("SRE!".to_string()).expect("OK");
                    sender
                        .send(SubmitGameEvent(run_to_submit.0.clone()))
                        .expect("Couldn't use the send channel");
                    client.compiled_run = Some((run_to_submit.0.clone(), true));
                }
            }
        });

        Self { join_handle }
    }
}

pub struct UiThread {}

impl UiThread {
    pub fn create_and_start(latest_data: ArcRw<StatsBlockWithFrames>, logs: ArcRw<Vec<String>>, connected: ArcRw<Conn>) {
        let mut term = crate::ui::create_term();
        let tick_duration = Duration::from_secs_f32(1. / 12.);
        term.clear().expect("Couldn't clear terminal");
        let cfg = config::CONFIG.with(|e| e.clone());
        thread::spawn(move || loop {
            let start_time = Instant::now();
            let read_data = latest_data.read().expect("Couldn't read last data");
            let log_list = logs.read().expect("Poisoned logs!").clone();

            term.draw(|f| {
                let mut layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(100)])
                    .split(f.size());

                if !connected.read().expect("AAA").is_ok {
                    draw_levi(f, layout[0]);
                    return;
                }

                if !cfg.ui_conf.hide_logo {
                    layout = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Min(12), Constraint::Percentage(100)])
                        .split(f.size());

                    crate::ui::draw_logo(f, layout[0]);
                }

                let mut info = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(100)])
                    .horizontal_margin(0)
                    .vertical_margin(0)
                    .split(layout[layout.len() - 1]);

                if !cfg.ui_conf.hide_logs {
                    info = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Min(28), Constraint::Percentage(100)])
                        .horizontal_margin(0)
                        .vertical_margin(0)
                        .split(layout[layout.len() - 1]);

                    crate::ui::draw_logs(f, info[0], &log_list);
                }

                crate::ui::draw_info_table(f, info[info.len() - 1], &read_data);

                let delay = if start_time.elapsed() > tick_duration {
                    Duration::ZERO
                } else {
                    Instant::now() - start_time
                };
                thread::sleep(tick_duration - delay);
            })
            .unwrap();
        });
    }
}

type ArcRw<T> = Arc<RwLock<T>>;
