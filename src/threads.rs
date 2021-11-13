//
//  threads.rs - Management of threads
//

// Rewrite Counter:
// I HATE WINDOWS
// I HATE WINDOWS

use crate::{client::{ClientSharedState, ConnectionState, GamePollClient, SubmitGameEvent}, config::cfg, grpc_client::GameSubmissionClient, socketio_client::LiveGameClient, ui::UiThread, websocket_server::WebsocketServer};
use std::{sync::Arc, time::{Duration, Instant, UNIX_EPOCH}};
use ddcore_rs::models::StatsBlockWithFrames;
use tokio::sync::{
    mpsc::{channel, Receiver, Sender},
    RwLock,
};

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
            GameSubmissionClient::init(sge_recv, log_send.clone(), ssio_send.clone()).await;
            WebsocketServer::init(last_poll.clone(), color_edit.clone(), current_snowflake.clone(), replay_request_send.clone(), conn.clone()).await;
            LiveGameClient::init(conn.clone(), last_poll.clone(), ssio_recv).await;
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
