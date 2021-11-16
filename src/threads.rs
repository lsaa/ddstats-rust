//
//  threads.rs - Management of threads
//

// Rewrite Counter:
// I HATE WINDOWS
// I HATE WINDOWS

use crate::{client::{ClientSharedState, ConnectionState, GamePollClient, SubmitGameEvent}, config::cfg, consts, grpc_client::GameSubmissionClient, socketio_client::LiveGameClient, ui::UiThread, websocket_server::WebsocketServer};
use std::{sync::Arc, time::{Duration, Instant, UNIX_EPOCH}};
use ddcore_rs::models::{GameStatus, StatsBlockWithFrames};
use discord_rich_presence::{DiscordIpc, activity::{self, Assets}, new_client};
use lazy_static::lazy_static;
use serde::Serialize;
use tokio::sync::{OnceCell, RwLock, mpsc::{channel, Receiver, Sender}};

lazy_static! {
    static ref PLAYER_LB_DATA: OnceCell<ddcore_rs::ddinfo::models::Entry> = OnceCell::const_new();
}

pub struct MainTask {
    state: MainTaskState,
}

pub struct MainTaskState {
    pub log_send: Sender<String>,
    pub log_recv: Receiver<String>,
    pub sge_send: Sender<SubmitGameEvent>,
    pub conn: Arc<RwLock<ConnectionState>>,
    pub last_poll: Arc<RwLock<StatsBlockWithFrames>>,
    pub logs: Arc<RwLock<Vec<String>>>,
}

#[derive(Debug, Serialize, Clone)]
pub struct WsBroadcast {
    #[serde(rename = "type")]
    pub _type: String,
    pub data: String,
}

impl MainTask {
    #[rustfmt::skip]
    pub async fn init() {
        let (log_send, log_recv) = channel(10);
        let (sge_send, sge_recv) = channel(3);
        let (replay_request_send, replay_request_recv) = channel(3);
        let conn: Arc<RwLock<ConnectionState>> = Arc::new(RwLock::default());
        let last_poll: Arc<RwLock<StatsBlockWithFrames>> = Arc::new(RwLock::default());
        let current_snowflake: Arc<RwLock<u128>> = Arc::new(RwLock::new(std::time::SystemTime::now().duration_since(UNIX_EPOCH).expect("error").as_millis()));
        let logs: Arc<RwLock<Vec<String>>> = Arc::new(RwLock::default());
        let color_edit: Arc<RwLock<crate::config::Styles>> = Arc::new(RwLock::default());
        let (exit_send, exit_recv) = tokio::sync::broadcast::channel(3);
        let (ssio_send, ssio_recv) = channel(3);
        let config = cfg();
        let (ws_broadcaster_send, _ws_broadcaster_recv) = tokio::sync::broadcast::channel::<WsBroadcast>(16);

        let mut main_task = MainTask {
            state: MainTaskState {
                log_send: log_send.clone(),
                log_recv,
                sge_send: sge_send.clone(),
                conn: conn.clone(),
                last_poll: last_poll.clone(),
                logs: logs.clone()
            },
        };

        GamePollClient::init(ClientSharedState {
            log_sender: log_send.clone(),
            connection_sender: conn.clone(),
            sge_sender: sge_send,
            last_poll: last_poll.clone(),
            snowflake: current_snowflake.clone(),
            replay_request: replay_request_recv,
        }).await;

        if config.ui_conf.enabled {
            UiThread::init(
                last_poll.clone(), 
                logs.clone(),
                conn.clone(),
                exit_send.clone(),
                color_edit.clone(),
                replay_request_send.clone()
            ).await;
        }

        if !config.offline {
            log::info!("ONLINE MODE!");
            GameSubmissionClient::init(sge_recv, log_send.clone(), ssio_send.clone(), ws_broadcaster_send.clone()).await;
            WebsocketServer::init(
                last_poll.clone(), 
                color_edit.clone(), 
                current_snowflake.clone(), 
                replay_request_send.clone(), 
                conn.clone(),
                ws_broadcaster_send.clone()
            ).await;
            LiveGameClient::init(conn.clone(), last_poll.clone(), ssio_recv).await;

            let conn_rpc = conn.clone();
            tokio::spawn(async move {
                let mut looper = tokio::time::interval(Duration::from_secs(1));
                let mut client = new_client("897951249507450880").expect("Can't go tits up");
                let mut is_rpc_connected = false;
                let mut tries = 0;

                loop {
                    looper.tick().await;
                    let connection = conn_rpc.read().await.clone();
                    let game_data = last_poll.read().await;

                    if !PLAYER_LB_DATA.initialized() && connection == ConnectionState::Connected && tries < 15 && game_data.block.player_id != 0 {
                        tries += 1;
                        if let Ok(player_entry) = ddcore_rs::ddinfo::get_leaderboard_user_by_id(game_data.block.player_id).await {
                            let _ = PLAYER_LB_DATA.set(player_entry);
                        }
                    }

                    let mut dagger = "pleb";

                    if PLAYER_LB_DATA.initialized() {
                        let time = (PLAYER_LB_DATA.get().unwrap().time as f32) / 10000.;
                        log::info!("{}", time);
                        if time >= 1000.0 {
                            dagger = "levi";
                        } else if time >= 500.0 {
                            dagger = "devil";
                        } else if time >= 250.0 {
                            dagger = "gold";
                        } else if time >= 120.0 {
                            dagger = "silver";
                        } else if time >= 60.0 {
                            dagger = "bronze";
                        }
                    }

                    if !is_rpc_connected && connection == ConnectionState::Connected {
                        if client.connect().is_ok() {
                            is_rpc_connected = true;
                            continue;
                        } else {
                            log::info!("{:?}", client.connect().err());
                        }
                    }

                    if is_rpc_connected && connection != ConnectionState::Connected {
                        if client.close().is_ok() {
                            is_rpc_connected = false;
                            continue;
                        }
                    }

                    if !is_rpc_connected { continue; }

                    if game_data.block.status() == GameStatus::Dead {


                        let death_type = consts::DEATH_TYPES.get(game_data.block.death_type as usize).unwrap();
                        let last_frame = game_data.frames.last().unwrap();
                        let last_frame_homers = last_frame.homing;
                        if last_frame.level_gems == 71 {
                            let _ = client.set_activity(activity::Activity::new()
                                .state(&format!("{} | {} at {:.4}s", death_type, last_frame_homers, game_data.block.time + game_data.block.starting_time))
                                .details(&format!("{} Gems ({} Lost)", game_data.block.gems_collected, game_data.block.gems_eaten + game_data.block.gems_despawned))
                                .assets(Assets::new()
                                .large_image(dagger)
                                .large_text("Playing V3")
                                .small_image("homing_colored")
                                .small_text(&format!("{} Homing", last_frame_homers)))
                            );
                        } else if last_frame.level_gems == 70 {
                            let _ = client.set_activity(activity::Activity::new()
                                .state(&format!("{} | {} LVL3 at {:.4}s", death_type, last_frame_homers, game_data.block.time + game_data.block.starting_time))
                                .details(&format!("{} Gems ({} Lost)", game_data.block.gems_collected, game_data.block.gems_eaten + game_data.block.gems_despawned))
                                .assets(Assets::new()
                                .large_image(dagger)
                                .large_text("Playing V3")
                                .small_image("homing_colored")
                                .small_text(&format!("{} Homing", last_frame_homers)))
                            );
                        } else if last_frame.level_gems >= 10 {
                            let _ = client.set_activity(activity::Activity::new()
                                .state(&format!("{} | Level 2 at {:.4}s", death_type, game_data.block.time + game_data.block.starting_time))
                                .details(&format!("{} Gems ({} Lost)", game_data.block.gems_collected, game_data.block.gems_eaten + game_data.block.gems_despawned))
                                .assets(Assets::new()
                                .large_image(dagger)
                                .large_text("Playing V3"))
                            );
                        } else {
                            let _ = client.set_activity(activity::Activity::new()
                                .state(&format!("{} | Level 1 at {:.4}s", death_type, game_data.block.time + game_data.block.starting_time))
                                .details(&format!("{} Gems ({} Lost)", game_data.block.gems_collected, game_data.block.gems_eaten + game_data.block.gems_despawned))
                                .assets(Assets::new()
                                .large_image(dagger)
                                .large_text("Playing V3"))
                            );
                        }
                    } else if game_data.block.is_replay {
                        if game_data.block.level_gems == 71 {
                            let _ = client.set_activity(activity::Activity::new()
                                .state(&format!("Replay | {} at {:.4}s", game_data.block.homing, game_data.block.time + game_data.block.starting_time))
                                .details(&format!("{} Gems ({} Lost)", game_data.block.gems_collected, game_data.block.gems_eaten + game_data.block.gems_despawned))
                                .assets(Assets::new()
                                .large_image(dagger)
                                .large_text("Playing V3")
                                .small_image("homing_colored")
                                .small_text(&format!("{} Homing", game_data.block.homing)))
                            );
                        } else if game_data.block.level_gems == 70 {
                            let _ = client.set_activity(activity::Activity::new()
                                .state(&format!("Replay | {} LVL3 at {:.4}s", game_data.block.homing, game_data.block.time + game_data.block.starting_time))
                                .details(&format!("{} Gems ({} Lost)", game_data.block.gems_collected, game_data.block.gems_eaten + game_data.block.gems_despawned))
                                .assets(Assets::new()
                                .large_image(dagger)
                                .large_text("Playing V3")
                                .small_image("homing_colored")
                                .small_text(&format!("{} Homing", game_data.block.homing)))
                            );
                        } else if game_data.block.level_gems >= 10 {
                            let _ = client.set_activity(activity::Activity::new()
                                .state(&format!("Replay | Level 2 at {:.4}s", game_data.block.time + game_data.block.starting_time))
                                .details(&format!("{} Gems ({} Lost)", game_data.block.gems_collected, game_data.block.gems_eaten + game_data.block.gems_despawned))
                                .assets(Assets::new()
                                .large_image(dagger)
                                .large_text("Playing V3"))
                            );
                        } else {
                            let _ = client.set_activity(activity::Activity::new()
                                .state(&format!("Replay | Level 1 at {:.4}s", game_data.block.time + game_data.block.starting_time))
                                .details(&format!("{} Gems ({} Lost)", game_data.block.gems_collected, game_data.block.gems_eaten + game_data.block.gems_despawned))
                                .assets(Assets::new()
                                .large_image(dagger)
                                .large_text("Playing V3"))
                            );
                        }
                    } else {
                        if game_data.block.level_gems == 71 {
                            let _ = client.set_activity(activity::Activity::new()
                                .state(&format!("{} at {:.4}s", game_data.block.homing, game_data.block.time + game_data.block.starting_time))
                                .details(&format!("{} Gems ({} Lost)", game_data.block.gems_collected, game_data.block.gems_eaten + game_data.block.gems_despawned))
                                .assets(Assets::new()
                                .large_image(dagger)
                                .large_text("Playing V3")
                                .small_image("homing_colored")
                                .small_text(&format!("{} Homing", game_data.block.homing)))
                            );
                        } else if game_data.block.level_gems == 70 {
                            let _ = client.set_activity(activity::Activity::new()
                                .state(&format!("{} LVL3 at {:.4}s", game_data.block.homing, game_data.block.time + game_data.block.starting_time))
                                .details(&format!("{} Gems ({} Lost)", game_data.block.gems_collected, game_data.block.gems_eaten + game_data.block.gems_despawned))
                                .assets(Assets::new()
                                .large_image(dagger)
                                .large_text("Playing V3")
                                .small_image("homing_colored")
                                .small_text(&format!("{} Homing", game_data.block.homing)))
                            );
                        } else if game_data.block.level_gems >= 10 {
                            let _ = client.set_activity(activity::Activity::new()
                                .state(&format!("Level 2 at {:.4}s", game_data.block.time + game_data.block.starting_time))
                                .details(&format!("{} Gems ({} Lost)", game_data.block.gems_collected, game_data.block.gems_eaten + game_data.block.gems_despawned))
                                .assets(Assets::new()
                                .large_image(dagger)
                                .large_text("Playing V3"))
                            );
                        } else {
                            let _ = client.set_activity(activity::Activity::new()
                                .state(&format!("Level 1 at {:.4}s", game_data.block.time + game_data.block.starting_time))
                                .details(&format!("{} Gems ({} Lost)", game_data.block.gems_collected, game_data.block.gems_eaten + game_data.block.gems_despawned))
                                .assets(Assets::new()
                                .large_image(dagger)
                                .large_text("Playing V3"))
                            );
                        }
                   }
                }
            });
        }

        main_task.run(exit_recv).await;
    }

    pub async fn run(&mut self, mut exit_message: tokio::sync::broadcast::Receiver<bool>) {
        let mut interval = tokio::time::interval(Duration::from_secs_f32(1. / 3.));
        loop {
            interval.tick().await;
            log::info!("MAIN  TICK {:?}", Instant::now());
            tokio::select! {
                new_log = self.state.log_recv.recv() => self.handle_log(new_log.unwrap()).await,
                _msg = exit_message.recv() => break,
            };
        }
    }

    pub async fn handle_log(&mut self, new_log: String) {
        self.state.logs.write().await.push(new_log);
    }
}
