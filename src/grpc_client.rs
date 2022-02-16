//
//  grpc_client.rs - I hate GRPC
//

use crate::{client::SubmitGameEvent, consts::{SUBMIT_RETRY_MAX, V3_SURVIVAL_HASH}, socketio_client::SubmitSioEvent, threads::{State, Message, AAS}, websocket_server::WsBroadcast};
use clipboard::{ClipboardContext, ClipboardProvider};

pub struct GameSubmissionClient;

impl GameSubmissionClient {
    pub async fn init(state: AAS<State>) {
        tokio::spawn(async move {
            use crate::grpc_models::game_recorder_client::GameRecorderClient;
            use crate::grpc_models::{ClientStartRequest, SubmitGameRequest};
            let cfg = crate::config::cfg();
            let mut client = GameRecorderClient::connect(cfg.grpc_host.clone()).await.expect("No Connection");
            let _res = client.client_start(ClientStartRequest { version: "0.6.10".to_owned() }).await;
            let mut bus_recv = state.load().msg_bus.0.subscribe();

            loop {
                let state = state.load();
                tokio::select! {
                    msg = bus_recv.recv() => if let Ok(Message::SubmitGame(sge)) = msg {
                        log::info!("Got submit request");
                        if !should_submit(&sge) { continue; }

                        let mut res = client.submit_game(SubmitGameRequest::from_compiled_run(sge.0.clone())).await;
                        for _i in 0..SUBMIT_RETRY_MAX {
                            if res.is_ok() { break; }
                            res = client.submit_game(SubmitGameRequest::from_compiled_run(sge.0.clone())).await;
                        }
        
                        if res.is_ok() {
                            let res = res.as_ref().unwrap().get_ref();
        
                            let _ = state.msg_bus.0.send(Message::WebSocketMessage(WsBroadcast {
                                _type: "ddstats_game_submit".into(),
                                data: format!("{{ \"game_id\": {}, \"snowflake\": {} }}", res.game_id, sge.1)
                            }));

                            let _ = state.msg_bus.0.send(Message::Log(format!("Submitted {}", res.game_id)));
        
                            if cfg.auto_clipboard {
                                let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                                let new_clip = format!("{}/games/{}", cfg.host, res.game_id);
                                ctx.set_contents(new_clip).unwrap();
                            }
        
                            if should_submit_sio(&sge) {
                                let _ = state.msg_bus.0.send(Message::SocketIoMessage(SubmitSioEvent { game_id: res.game_id }));
                            }

                            let should_upload = should_upload_replay(&sge);
                            let game_id = res.game_id;
                            let data_arc = sge.2.clone();
                            tokio::spawn(async move {
                                let replay_hash = format!("{:x}", ddcore_rs::md5::compute(&*data_arc));
                                if ddcore_rs::ddreplay::create_ddstats_trace(game_id as u64, replay_hash).await.is_ok() {
                                    log::info!("traced ddstats game: {}", game_id);
                                    if should_upload {
                                        let _ = state.msg_bus.0.send(Message::UploadReplayData(data_arc, false));
                                    }
                                }
                            });
                        } else {
                            log::error!("Couldn't submit: {:?}", res);
                            let _ = state.msg_bus.0.send(Message::Log("Failed to Submit".to_string()));
                        }
                    },
                };
            }
        });
    }
}

#[rustfmt::skip]
fn should_submit_sio(data: &SubmitGameEvent) -> bool {
    let cfg = crate::config::cfg();
    let is_non_default = data.0.level_hash_md5.ne(&V3_SURVIVAL_HASH.to_uppercase());
    if is_non_default && !cfg.submit.non_default_spawnsets { return false; }
    cfg.stream.stats && !data.0.is_replay
    || cfg.stream.replay_stats && data.0.is_replay
}

#[rustfmt::skip]
fn should_upload_replay(data: &SubmitGameEvent) -> bool{
    let cfg = crate::config::cfg();
    if !cfg.upload_replays_automatically { return false; }
    let is_non_default = data.0.level_hash_md5.ne(&V3_SURVIVAL_HASH.to_uppercase());
    if is_non_default { return false; }
    if data.0.time_max < 100. { return false; }
    if data.0.time_max < 500. && data.0.daggers_hit > 0 { return false; }
    if data.0.is_replay && !cfg.submit.replay_stats { return false; }
    if !data.0.is_replay && !cfg.submit.stats { return false; }
    true
}

#[rustfmt::skip]
fn should_submit(data: &SubmitGameEvent) -> bool{
    let cfg = crate::config::cfg();
    let is_non_default = data.0.level_hash_md5.ne(&V3_SURVIVAL_HASH.to_uppercase());
    if is_non_default && !cfg.submit.non_default_spawnsets { return false; }
    if data.0.is_replay && !cfg.submit.replay_stats { return false; }
    if !data.0.is_replay && !cfg.submit.stats { return false; }
    true
}
