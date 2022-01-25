//
//  discord.rs -Rich Presence Thread
//

use std::{time::Duration};
use ddcore_rs::models::GameStatus;
use discord_rich_presence::{new_client, activity::{self, Assets}, DiscordIpc};
use lazy_static::lazy_static;
use tokio::sync::OnceCell;
use crate::{threads::{State, AAS}, client::ConnectionState, consts};

lazy_static! {
    static ref PLAYER_LB_DATA: OnceCell<ddcore_rs::ddinfo::models::Entry> = OnceCell::const_new();
}

pub struct RichPresenceClient;

impl RichPresenceClient {
    pub async fn init(state: AAS<State>) {
        tokio::spawn(async move {
            let mut looper = tokio::time::interval(Duration::from_secs(1));
            let mut client = new_client("897951249507450880").expect("Can't go tits up");
            let mut is_rpc_connected = false;
            let mut tries = 0;

            loop {
                looper.tick().await;
                let state = state.load();
                let ref game_data = state.last_poll;

                if !PLAYER_LB_DATA.initialized() && *state.conn == ConnectionState::Connected && tries < 15 && game_data.block.player_id != 0 {
                    tries += 1;
                    match ddcore_rs::ddinfo::get_leaderboard_user_by_id(game_data.block.player_id).await {
                        Ok(player_entry) => { let _ = PLAYER_LB_DATA.set(player_entry); },
                        Err(e) => { log::warn!("Failed to pull player data from ddinfo ({}/{}): {:?}", tries, 15, e); }
                    }
                }

                let mut dagger = "pleb";

                if PLAYER_LB_DATA.initialized() {
                    let time = PLAYER_LB_DATA.get().unwrap().time;
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

                if !is_rpc_connected && *state.conn == ConnectionState::Connected {
                    if client.connect().is_ok() {
                        is_rpc_connected = true;
                        log::info!("Connected discord rich presence");
                        continue;
                    }
                }

                if is_rpc_connected && *state.conn != ConnectionState::Connected {
                    if client.close().is_ok() {
                        is_rpc_connected = false;
                        log::info!("Disconnected discord rich presence");
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
}
