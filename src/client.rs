use std::{
    sync::mpsc::Sender,
    thread,
    time::{Duration, Instant},
};

use crate::{
    config,
    consts::{DD_PROCESS, VERSION},
    mem::{GameConnection, StatsFrame},
};

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

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
    pub last_game_state: GameStatus,
    pub submitted_data: bool,
    pub compiled_run: Option<CompiledRun>,
    pub log_sender: Sender<String>,
    pub conn: (Sender<bool>, Sender<bool>),
    pub connecting_start: Instant,
    pub sender: Sender<SubmitGameEvent>,
}

impl Client {
    pub fn game_loop(&mut self) {
        match self.game_state {
            GameClientState::NotConnected => self.not_connected(),
            GameClientState::Connecting => self.connecting(),
            GameClientState::Connected => self.connected(),
        }
    }

    fn resolve_connection(&mut self) -> bool {
        if !self.game_connection.is_alive() {
            self.game_state = GameClientState::NotConnected;
            self.log_sender
                .send("Game Disconnected!".to_string())
                .expect("Can't access log");
            self.conn.1.send(false).unwrap();
            return false;
        }
        return true;
    }

    fn not_connected(&mut self) {
        if let Ok(game) = GameConnection::try_create(DD_PROCESS) {
            self.game_state = GameClientState::Connecting;
            self.game_connection = game;
            self.connecting_start = Instant::now()
        } else {
            log::info!("Coudln't connect;");
            thread::sleep(Duration::from_secs(3));
        }
    }

    fn connecting(&mut self) {
        if !self.resolve_connection() {
            return;
        }

        if self.connecting_start.elapsed() > Duration::from_secs(2) {
            self.game_state = GameClientState::NotConnected;
            self.game_connection = GameConnection::dead_connection();
        }

        match self.game_connection.read_stats_block() {
            Ok(_) => {
                self.game_state = GameClientState::Connected;
                self.log_sender
                    .send("Game Connected!".to_owned())
                    .expect("Can't access log");
                self.conn.0.send(true).unwrap();
            }
            Err(_e) => {}
        }
    }

    fn connected(&mut self) {
        if !self.resolve_connection() {
            return;
        }

        let _cfg = config::CONFIG.with(|z| z.clone());

        let with_frames = self.game_connection.read_stats_block_with_frames();
        if let Ok(with_frames) = with_frames {
            let last_frame = with_frames.frames.last();
            if last_frame.is_none() {
                return;
            }
            let last_frame = last_frame.unwrap();
            let status: GameStatus = FromPrimitive::from_i32(with_frames.block.status).unwrap();
            let old = self.last_game_state;

            if status == GameStatus::Playing
                || (old != GameStatus::OtherReplay && status == GameStatus::OtherReplay)
                || (old != GameStatus::OwnReplayFromLeaderboard
                    && status == GameStatus::OwnReplayFromLeaderboard)
            {
                self.submitted_data = false;
            }

            if status == GameStatus::Dead && old != GameStatus::Dead {
                // YOU DIED HHAHAHAHAHA
                log::info!(
                    "DEATH {} {}",
                    self.submitted_data,
                    with_frames.block.stats_finished_loading
                );
            }

            if with_frames.block.stats_finished_loading && !self.submitted_data {
                if status == GameStatus::Dead
                    || status == GameStatus::OtherReplay
                    || status == GameStatus::OwnReplayFromLeaderboard
                {
                    let mut player_id = with_frames.block.player_id;
                    let mut replay_player_id = with_frames.block.replay_player_id;

                    if with_frames.block.is_replay {
                        player_id = with_frames.block.replay_player_id;
                        replay_player_id = with_frames.block.player_id;
                    }
                    /*
                    if (with_frames.block.is_replay && !cfg.submit.replay_stats)
                        || (!with_frames.block.is_replay && !cfg.submit.stats)
                        || (!with_frames.block.level_hash().eq(V3_SURVIVAL_HASH)
                            && !cfg.submit.non_default_spawnsets)
                    {
                        self.last_game_state = status;
                        return;
                    }
                    */
                    log::info!("I am submit?");
                    self.compiled_run = Some(CompiledRun {
                        version: VERSION.to_owned(),
                        player_id,
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
                        replay_player_id,
                        per_enemy_alive_count: last_frame.per_enemy_alive_count.clone(),
                        per_enemy_kill_count: last_frame.per_enemy_kill_count.clone(),
                        time_max: with_frames.block.time_max,
                        gems_collected: last_frame.gems_collected,
                        gems_total: last_frame.gems_total,
                        gems_despawned: last_frame.gems_despawned,
                        gems_eaten: last_frame.gems_eaten,
                        daggers_eaten: last_frame.daggers_eaten,
                        daggers_fired: last_frame.daggers_fired,
                        daggers_hit: last_frame.daggers_hit,
                        enemies_killed: with_frames.block.kills,
                        enemies_alive: last_frame.enemies_alive,
                        level_gems: last_frame.level_gems,
                        homing_daggers: last_frame.homing,
                        stats: with_frames.frames,
                    });

                    self.sender
                        .send(SubmitGameEvent(self.compiled_run.as_ref().unwrap().clone()))
                        .expect("FUNNY SENDER");

                    self.submitted_data = true;
                }
            }

            self.last_game_state = status;
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CompiledRun {
    pub version: String,
    pub player_id: i32,
    pub player_name: String,
    pub level_hash_md5: String,
    pub time_max: f32,
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
    pub gems_collected: i32,
    pub enemies_killed: i32,
    pub enemies_alive: i32,
    pub level_gems: i32,
    pub homing_daggers: i32,
    pub gems_total: i32,
    pub gems_despawned: i32,
    pub gems_eaten: i32,
    pub daggers_eaten: i32,
    pub daggers_hit: i32,
    pub daggers_fired: i32,
    pub per_enemy_alive_count: [i16; 17],
    pub per_enemy_kill_count: [i16; 17],
    pub stats: Vec<StatsFrame>,
}

#[derive(FromPrimitive, Debug, PartialEq, Clone, Copy)]
pub enum GameStatus {
    Title = 0,
    Menu,
    Lobby,
    Playing,
    Dead,
    OwnReplayFromLastRun,
    OwnReplayFromLeaderboard,
    OtherReplay,
}

pub struct SubmitGameEvent(pub CompiledRun);
