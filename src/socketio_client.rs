//
// SocketIO Client - socketio_client.rs
//

use std::time::{Duration, Instant};
use ddcore_rs::models::{GameStatus, StatsBlockWithFrames};
use native_tls::{Protocol, TlsConnector};
use anyhow::Result;
use num_traits::FromPrimitive;
use websocket::{ClientBuilder, Message, sync::{Client, stream::NetworkStream}};
use crate::{client::ConnectionState, threads::{AAS, State}};

/////////////////////////////////

#[derive(Debug, PartialEq, PartialOrd)]
pub enum SioStatus {
	Disconnected = 1,
	Connecting,
	Connected,
	LoggedIn,
	Timeout,
}

pub struct LiveGameClient {
    pub sio_status: SioStatus,
}

#[derive(Clone)]
pub struct SubmitSioEvent {
    pub game_id: u32,
}

/////////////////////////////////

impl LiveGameClient {
    pub async fn init(state: AAS<State>) {
        tokio::spawn(async move {
            let cfg = crate::config::cfg();
            let mut msg_bus = state.load().msg_bus.0.subscribe();
            let mut lgc = LiveGameClient { sio_status: SioStatus::Disconnected };
            let mut current_socket = None;
            let mut sio_tick_interval = tokio::time::interval(Duration::from_secs_f32(1. / 3.));
            let mut last_ping = Instant::now();
            let mut new_sio_message = None;
            let mut login_cooldown = Instant::now();

            loop {

                tokio::select! {
                    msg = msg_bus.recv() => match msg {
                        Ok(crate::threads::Message::SocketIoMessage(data)) => {
                            new_sio_message = Some(data);
                        },
                        _ => {},
                    },
                    _elapsed = sio_tick_interval.tick() => {
                        let state = state.load();
                        let ref last_data = state.last_poll;
                        if (*state.conn).eq(&ConnectionState::Connected) {
                            if lgc.sio_status.ne(&SioStatus::LoggedIn) {
                                if Instant::now().duration_since(login_cooldown) > Duration::from_secs(2) {
                                    login_cooldown = Instant::now();
                                    let connection = connect();
                                    if connection.is_ok() {
                                        current_socket = Some(connection.unwrap());
                                        lgc.sio_status = SioStatus::Connected;
                                        let res = current_socket.as_mut().unwrap().send_message(&Message::text(create_login_message(last_data.block.player_id)));
                                        if res.is_ok() {
                                            lgc.sio_status = SioStatus::LoggedIn;
                                        }
                                    }
                                }
                            } else {
                                if let Some(submit_evt) = &new_sio_message {
                                    let mut notify_pb = cfg.discord.notify_player_best;
                                    let mut notify_above_1000 = cfg.discord.notify_above_1000;

                                    if last_data.block.is_replay {
                                        notify_pb = false;
                                        notify_above_1000 = false;
                                    }

                                    log::info!("Submitting SIO {}", create_sio_submit(&submit_evt, (notify_pb, notify_above_1000)));
                                    let sio_submit = current_socket.as_mut().unwrap().send_message(&Message::text(create_sio_submit(&submit_evt, (notify_pb, notify_above_1000))));
                                    new_sio_message = None;
                                    if sio_submit.is_err() {
                                        current_socket = None;
                                        lgc.sio_status = SioStatus::Disconnected;
                                        continue;
                                    }
                                }

                                // KeepAlive
                                if last_ping.elapsed() > Duration::from_secs(24) {
                                    let ping = current_socket.as_mut().unwrap().send_message(&Message::text("2"));
                                    last_ping = Instant::now();
                                    if ping.is_err() {
                                        current_socket = None;
                                        lgc.sio_status = SioStatus::Disconnected;
                                        continue;
                                    }
                                }

                                if last_data.block.is_in_game || last_data.block.status == GameStatus::Dead as i32 {
                                    let mut death_type = -2;
                                    if last_data.block.status == GameStatus::Playing as i32 {
                                        death_type = -1;
                                    } else if last_data.block.status == GameStatus::Dead as i32 {
                                        death_type = last_data.block.death_type as i32;
                                    }

                                    if should_submit_sio(&last_data) {
                                        let res = current_socket.as_mut().unwrap().send_message(&Message::text(create_submit_stats_message(&last_data, death_type)));
                                        if res.is_err() {
                                            current_socket = None;
                                            lgc.sio_status = SioStatus::Disconnected;
                                            continue;
                                        }
                                    }
                                } else {
                                    let status: GameStatus = FromPrimitive::from_i32(last_data.block.status).unwrap();
                                    let sio_status = match status {
                                        GameStatus::Title | GameStatus::Menu => 4,
                                        GameStatus::Lobby => 5,
                                        GameStatus::Playing => 2,
                                        GameStatus::Dead => 6,
                                        _ => 3,
                                    };
                                    if cfg.stream.stats {
                                        let res = current_socket.as_mut().unwrap().send_message(&Message::text(create_change_status_message(last_data.block.player_id, sio_status)));
                                        if res.is_err() {
                                            current_socket = None;
                                            lgc.sio_status = SioStatus::Disconnected;
                                            continue;
                                        }
                                    }

                                }
                            }
                        } else {
                            current_socket = None;
                            lgc.sio_status = SioStatus::Disconnected;
                        }
                    }
                    }
                };
        });
    }
}

#[rustfmt::skip]
fn should_submit_sio(data: &StatsBlockWithFrames) -> bool {
    use crate::consts::V3_SURVIVAL_HASH;
    let cfg = crate::config::cfg();
    let is_non_default = data.block.level_hash().ne(&V3_SURVIVAL_HASH.to_uppercase());
    if is_non_default && !cfg.submit.non_default_spawnsets { return false; }
    cfg.stream.stats && !data.block.is_replay
    || cfg.stream.replay_stats && data.block.is_replay
}

fn create_sio_submit(ev: &SubmitSioEvent, (notify_pb, notify_above_1000): (bool, bool)) -> String {
    format!("42[\"game_submitted\",{},{},{}]", ev.game_id,notify_pb, notify_above_1000)
}

fn create_change_status_message(player_id: i32, status: i32) -> String {
    format!("42[\"status_update\",{},{}]", player_id, status)
}

fn create_submit_stats_message(data: &StatsBlockWithFrames, death: i32) -> String {
    let cfg = crate::config::cfg();
    format!("42[\"submit\",{},{:.4},{},{},{},{},{},{},{:.4},{:.4},{:.4},{},{},{},{}]",
        data.block.player_id,
        data.block.time,
        data.block.gems_total,
        data.block.homing,
        data.block.enemies_alive,
        data.block.kills,
        data.block.daggers_hit,
        data.block.daggers_fired,
        data.block.time_lvl2,
        data.block.time_lvl3,
        data.block.time_lvl4,
        data.block.is_replay,
        death,
        cfg.discord.notify_player_best,
        cfg.discord.notify_above_1000
    )
}

fn create_login_message(player_id: i32) -> String {
    format!("42[\"login\", {}]", player_id)
}

fn connect() -> Result<Client<Box<dyn NetworkStream + Send>>> {
    let tls_connector =  TlsConnector::builder()
        .min_protocol_version(Some(Protocol::Tlsv12))
        .max_protocol_version(Some(Protocol::Tlsv12))
        .build()?;

    ClientBuilder::new("wss://ddstats.com/socket.io/?EIO=3&transport=websocket")?
        .connect(Some(tls_connector))
        .map_err(|_| anyhow::Error::msg("Websocket Error"))
}
