//
//  grpc_client.rs - I hate GRPC
//

use crate::{client::SubmitGameEvent, consts::SUBMIT_RETRY_MAX};
use tokio::sync::mpsc::{Receiver, Sender};

pub struct GameSubmissionClient;

impl GameSubmissionClient {
    pub async fn init(mut sge_recv: Receiver<SubmitGameEvent>, log_sender: Sender<String>) {
        tokio::spawn(async move {
            use crate::grpc_models::game_recorder_client::GameRecorderClient;
            use crate::grpc_models::{ClientStartRequest, SubmitGameRequest};
            let cfg = crate::config::cfg();
            let mut client = GameRecorderClient::connect(cfg.grpc_host.clone())
                .await
                .expect("No Connection");
            let res = client
                .client_start(ClientStartRequest {
                    version: "0.6.8".to_owned(),
                })
                .await
                .expect("A");
            log::info!("MOTD: {}", res.get_ref().motd);
            while let Some(sge) = sge_recv.recv().await {
                log::info!("Got submit req");
                let mut res = client
                    .submit_game(SubmitGameRequest::from_compiled_run(sge.0.clone()))
                    .await;
                for _i in 0..SUBMIT_RETRY_MAX {
                    if res.is_ok() {
                        break;
                    }
                    res = client
                        .submit_game(SubmitGameRequest::from_compiled_run(sge.0.clone()))
                        .await;
                }

                if res.is_ok() {
                    log_sender
                        .send(format!("Submitted {}", res.unwrap().get_ref().game_id))
                        .await
                        .expect("AAA");
                } else {
                    log_sender
                        .send(format!("Failed to Submit"))
                        .await
                        .expect("AAA");
                }
            }
        });
    }
}
