//
//  client.rs - Game Poll
//

use crate::consts::*;
use ddcore_rs::ddinfo;
use ddcore_rs::ddinfo::ddcl_submit::DdclSecrets;
use ddcore_rs::memory::{ConnectionParams, GameConnection, MemoryOverride, OperatingSystem};
use ddcore_rs::models::{GameStatus, StatsBlockWithFrames, StatsFrame};
use num_traits::FromPrimitive;
use serde::Serialize;
use tokio::sync::OnceCell;
use tokio::sync::mpsc::Receiver;
use std::sync::Arc;
use std::task::Poll;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc::Sender, RwLock};
use tokio::time::{self, timeout};

static MARKER_ADDR: OnceCell<usize> = OnceCell::const_new();

#[derive(PartialEq, Debug, Clone, Serialize)]
pub enum ConnectionState {
    NotConnected,
    Connecting,
    Connected,
}

impl std::default::Default for ConnectionState {
    fn default() -> Self {
        ConnectionState::NotConnected
    }
}

pub struct ClientSharedState {
    pub log_sender: Sender<String>,
    pub connection_sender: Arc<RwLock<ConnectionState>>,
    pub sge_sender: Sender<SubmitGameEvent>,
    pub last_poll: Arc<RwLock<StatsBlockWithFrames>>,
    pub snowflake: Arc<RwLock<u128>>,
    pub replay_request: Receiver<Vec<u8>>,
}

pub struct GamePollClient {
    pub connection: GameConnection,
    pub connection_state: ConnectionState,
    pub submitted_data: bool,
    pub state: ClientSharedState,
    pub connecting_start: Instant,
    pub last_game_state: GameStatus,
}

impl GamePollClient {
    pub async fn init(state: ClientSharedState) {
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs_f32(1. / 36.));
            let mut c = Self {
                state,
                connection: GameConnection::dead_connection(),
                submitted_data: false,
                connecting_start: Instant::now(),
                connection_state: ConnectionState::NotConnected,
                last_game_state: GameStatus::Menu,
            };

            loop {
                interval.tick().await;
                c.tick().await;
            }
        });
    }

    pub async fn tick(&mut self) {
        match self.connection_state {
            ConnectionState::NotConnected => self.not_connected().await,
            ConnectionState::Connecting => self.connecting().await,
            ConnectionState::Connected => self.connected().await,
        };
    }

    async fn not_connected(&mut self) {
        let cfg = crate::config::cfg();
        let os = if cfg.use_linux_proton {
            OperatingSystem::LinuxProton
        } else {
            if cfg!(target_os = "linux") {
                OperatingSystem::Linux
            } else if cfg!(target_os = "winwdows") {
                OperatingSystem::Windows
            } else {
                OperatingSystem::Windows
            }
        };

        let conn_res = GameConnection::try_create(ConnectionParams {
            create_child: cfg.linux_restart_as_child,
            operating_system: os,
            overrides: MemoryOverride {
                process_name: cfg.process_name_override.clone(),
                block_marker: Some(MARKER_ADDR.get_or_init(|| { async {
                    if cfg.block_marker_override.is_some() {
                        return cfg.block_marker_override.unwrap().clone();
                    }

                    if let Ok(marker_response) = ddcore_rs::ddinfo::get_ddstats_memory_marker(ddinfo::get_os()).await {
                        return marker_response.value.clone();
                    }

                    if cfg!(target_os = "winwdows") {
                        crate::consts::WINDOWS_BLOCK_START
                    } else {
                        crate::consts::LINUX_BLOCK_START
                    }
                }}).await.clone()),
            },
        });

        if let Ok(new_connection) = conn_res {
            self.connection_state = ConnectionState::Connecting;
            self.connection = new_connection;
            self.connecting_start = Instant::now();
            *self.state.connection_sender.write().await = self.connection_state.clone();
        } else {
            log::info!("{:?}", conn_res.err());
            std::thread::sleep(Duration::from_secs(0));
        }
    }

    async fn connecting(&mut self) {
        if self.connecting_start.elapsed() > Duration::from_secs(3) {
            self.connection = GameConnection::dead_connection();
            self.connection_state = ConnectionState::NotConnected;
            *self.state.connection_sender.write().await = self.connection_state.clone();
        }

        if let Ok(_) = self.connection.read_stats_block_with_frames() {
            self.connection_state = ConnectionState::Connected;
            self.state
                .log_sender
                .send("Game Connected!".to_owned())
                .await
                .expect("Poisoned Logs");
            *self.state.connection_sender.write().await = self.connection_state.clone();
        }
    }

    async fn connected(&mut self) {
        if !self.resolve_connection().await {
            return;
        }

        if let Ok(data) = self.connection.read_stats_block_with_frames() {
            if let Some(snowflake) = self.new_snowflake(&data).await {
                self.state.snowflake.write().await.clone_from(&snowflake);
            }

            let waker = futures::task::noop_waker();
            let mut cx = std::task::Context::from_waker(&waker);
            if let Poll::Ready(Some(replay)) = self.state.replay_request.poll_recv(&mut cx) {
                let res = self.connection.play_replay(replay);
                if res.is_ok() {
                    self.connection.maximize_dd();
                }
            }

            if data.frames.last().is_none() {
                return;
            }

            let last = data.frames.last().unwrap();
            let status: GameStatus = data.block.status();
            let old = self.last_game_state;

            if GamePollClient::new_run_started(&status, &old) {
                self.submitted_data = false;
            }

            if self.should_submit(&data, &status) {
                log::info!("Attempting to submit run");
                let to_submit = GamePollClient::create_submit_event(&data, &last, self.state.snowflake.read().await.clone());
                self.submit_retry_until_success(to_submit).await;
                let c = data.clone();
                log::info!("{}", data.block.replay_buffer_length);
                if let Ok(replay) = self.connection.replay_bin() {
                    if c.block.status == 3 || c.block.status == 4 || c.block.status == 5 {
                        tokio::spawn(async move {
                            let _res = ddinfo::ddcl_submit::submit(&c, ddcl_secrets(), "ddstats-rust", "0.6.9.1", replay).await;
                        });
                    }
                }
                self.submitted_data = true;
            }

            self.last_game_state = status;
            let writer = timeout(Duration::from_secs_f32(1./120.), self.state.last_poll.write()).await;
            if let Ok(mut writer) = writer {
                writer.clone_from(&data);
            }
        }
    }

    async fn new_snowflake(&mut self, data: &StatsBlockWithFrames) -> Option<u128> {
        let status: GameStatus = FromPrimitive::from_i32(data.block.status).unwrap();
        let old = self.last_game_state;
        let snowflake = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let current = *self.state.snowflake.read().await;

        if old != GameStatus::OtherReplay && status == GameStatus::OtherReplay {
            return Some(snowflake);
        }

        if old != GameStatus::OwnReplayFromLeaderboard && status == GameStatus::OwnReplayFromLeaderboard {
            return Some(snowflake);
        }

        if old != GameStatus::OwnReplayFromLastRun && status == GameStatus::OwnReplayFromLastRun {
            return Some(snowflake);
        }

        if old != GameStatus::Menu && status == GameStatus::Menu {
            return Some(snowflake);
        }

        if old == GameStatus::Playing && status == GameStatus::Playing {
            if current > snowflake {
                return Some(snowflake);
            }
            if (snowflake - current) > (data.block.time * 1100.) as u128 {
                return Some(snowflake);
            }
        }

        None
    }

    async fn submit_retry_until_success(&mut self, event: SubmitGameEvent) {
        let mut res = self.state.sge_sender.send(event.clone()).await;
        loop {
            if res.is_ok() {
                break;
            }
            res = self.state.sge_sender.send(event.clone()).await;
        }
    }

    fn create_submit_event(data: &StatsBlockWithFrames, last: &StatsFrame, snowflake: u128) -> SubmitGameEvent {
        let mut player_id = data.block.player_id;
        let replay_player_id;

        if data.block.is_replay {
            player_id = data.block.replay_player_id;
            replay_player_id = data.block.player_id;
        } else {
            replay_player_id = 0;
        }

        SubmitGameEvent(CompiledRun {
            version: VERSION.to_owned(),
            player_id,
            player_name: data.block.player_username(),
            level_hash_md5: data.block.level_hash(),
            time_lvl2: data.block.time_lvl2,
            time_lvl3: data.block.time_lvl3,
            time_lvl4: data.block.time_lvl4,
            time_levi_down: data.block.levi_down_time,
            time_orb_down: data.block.orb_down_time,
            enemies_alive_max: data.block.enemies_alive_max,
            enemies_alive_max_time: data.block.time_enemies_alive_max,
            homing_daggers_max: data.block.max_homing,
            homing_daggers_max_time: data.block.time_max_homing,
            death_type: data.block.death_type as i32,
            is_replay: data.block.is_replay,
            replay_player_id,
            per_enemy_alive_count: last.per_enemy_alive_count.clone(),
            per_enemy_kill_count: last.per_enemy_kill_count.clone(),
            time_max: data.block.time_max,
            gems_collected: last.gems_collected,
            gems_total: last.gems_total,
            gems_despawned: last.gems_despawned,
            gems_eaten: last.gems_eaten,
            daggers_eaten: last.daggers_eaten,
            daggers_fired: last.daggers_fired,
            daggers_hit: last.daggers_hit,
            enemies_killed: last.kills,
            enemies_alive: last.enemies_alive,
            level_gems: last.level_gems,
            homing_daggers: last.homing,
            stats: data.frames.clone(),
        }, snowflake.clone())
    }

    #[rustfmt::skip]
    fn should_submit(&self, data: &StatsBlockWithFrames, status: &GameStatus) -> bool {
        let status = *status;

        data.block.stats_finished_loading
        && !self.submitted_data
        && (status == GameStatus::Dead
        || status == GameStatus::OtherReplay
        || status == GameStatus::OwnReplayFromLeaderboard)
    }

    #[rustfmt::skip]
    fn new_run_started(status: &GameStatus, old: &GameStatus) -> bool {
        let status = *status;
        let old = *old;

        status == GameStatus::Playing
        || (old != GameStatus::OtherReplay && status == GameStatus::OtherReplay)
        || (old != GameStatus::OwnReplayFromLeaderboard && status == GameStatus::OwnReplayFromLeaderboard)
    }

    async fn resolve_connection(&mut self) -> bool {
        if let Err(e) = self.connection.is_alive_res() {
            log::warn!("Disconnected: {:?}", e);
            log::info!("{:?}", self.connection.last_fetch);
            self.connection_state = ConnectionState::NotConnected;
            *self.state.connection_sender.write().await = self.connection_state.clone();
            return false;
        }
        return true;
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

#[rustfmt::skip]
fn ddcl_secrets() -> Option<DdclSecrets> {
    let iv = std::option_env!("DDCL_SECRETS_IV")?.to_owned();
    let pass = std::option_env!("DDCL_SECRETS_PASS")?.to_owned();
    let salt = std::option_env!("DDCL_SECRETS_SALT")?.to_owned();
    Some(DdclSecrets { iv, pass, salt })
}

#[derive(Clone)]
pub struct SubmitGameEvent(pub CompiledRun, pub u128);

