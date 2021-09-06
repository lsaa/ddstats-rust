use std::{process::exit, sync::mpsc::Sender, thread, time::{Duration, Instant}};

use crate::{
    consts::{DD_PROCESS, VERSION},
    mem::{GameConnection, StatsFrame},
};

#[derive(PartialEq, Debug)]
pub enum GameClientState {
    NotConnected,
    Connecting,
    Connected,
}

pub struct Client {
    pub game_connection: GameConnection,
    pub game_state: GameClientState,
    pub last_game_update: Instant,
    pub compiled_run: Option<(CompiledRun, bool)>,
    pub log_sender: Sender<String>,
}

impl Client {
    pub fn game_loop(&mut self) {
        if Instant::now().duration_since(self.last_game_update) > Duration::from_secs_f32(1. / 36.)
        {
            self.last_game_update = Instant::now();
            match self.game_state {
                GameClientState::NotConnected => self.not_connected(),
                GameClientState::Connecting => self.connecting(),
                GameClientState::Connected => self.connected(),
            }
        }
    }

    fn resolve_connection(&mut self) -> bool {
        if !self.game_connection.is_alive() {
            self.game_state = GameClientState::NotConnected;
            self.log_sender
                .send("Game Disconnected!".to_string())
                .expect("Can't access log");
            return false;
        }
        return true;
    }

    fn not_connected(&mut self) {
        if let Ok(game) = GameConnection::try_create(DD_PROCESS) {
            self.game_state = GameClientState::Connecting;
            self.game_connection = game;
        } else {
            thread::sleep(Duration::from_secs(3));
        }
    }

    fn connecting(&mut self) {
        if !self.resolve_connection() {
            return;
        }
        print!("AaA");
        exit(0);

        if let Ok(_) = self.game_connection.read_stats_block() {
            self.game_state = GameClientState::Connected;
            self.log_sender
                .send("Game Connected!".to_owned())
                .expect("Can't access log");
        }
    }

    fn connected(&mut self) {
        if !self.resolve_connection() {
            return;
        }
        let with_frames = self.game_connection.read_stats_block_with_frames();
        if let Ok(with_frames) = with_frames {
            if with_frames.block.status == 4 && self.compiled_run.is_none() {
                self.compiled_run = Some((
                    CompiledRun {
                        version: VERSION.to_owned(),
                        player_id: with_frames.block.player_id,
                        player_name: with_frames.block.player_username(),
                        level_hash_md5: with_frames.block.level_hash(),
                        time_lvl2: with_frames.block.time_lvl2,
                        time_lvl3: with_frames.block.time_lvl3,
                        time_lvl4: with_frames.block.time_lvl4,
                        time_levi_down: with_frames.block.levi_down_time,
                        time_orb_down: with_frames.block.orb_down_time,
                        enemies_alive_max: with_frames.block.enemies_alive_max,
                        enemies_alive_max_time: with_frames.block.time_enemies_alive_max,
                        homing_daggers_max: with_frames.block.max_homing,
                        homing_daggers_max_time: with_frames.block.time_max_homing,
                        death_type: with_frames.block.death_type as i32,
                        is_replay: with_frames.block.is_replay,
                        replay_player_id: with_frames.block.replay_player_id,
                        stats: with_frames.frames,
                    },
                    false,
                ));
            }

            if with_frames.block.status != 4 && self.compiled_run.is_some() {
                self.compiled_run = None;
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CompiledRun {
    // fuck you VHS for not making the server code public
    pub version: String,
    pub player_id: i32,
    pub player_name: String,
    pub level_hash_md5: String,
    pub time_lvl2: f32,
    pub time_lvl3: f32,
    pub time_lvl4: f32,
    pub time_levi_down: f32,
    pub time_orb_down: f32,
    pub enemies_alive_max: i32,
    pub enemies_alive_max_time: f32,
    pub homing_daggers_max: i32,
    pub homing_daggers_max_time: f32,
    pub death_type: i32,
    pub is_replay: bool,
    pub replay_player_id: i32,
    pub stats: Vec<StatsFrame>,
}

pub struct SubmitGameEvent(pub CompiledRun);
