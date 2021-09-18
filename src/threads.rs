//
//  threads.rs - Management of threads
//

// Rewrite Counter:
// I HATE WINDOWS
// I HATE WINDOWS

use crate::{
    client::{ClientSharedState, ConnectionState, GamePollClient, SubmitGameEvent},
    config::cfg,
    grpc_client::GameSubmissionClient,
    mem::StatsBlockWithFrames,
    ui::UiThread,
    websocket_server::WebsocketServer,
};
use std::{sync::Arc, time::Duration};
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
        let conn: Arc<RwLock<ConnectionState>> = Arc::new(RwLock::default());
        let last_poll: Arc<RwLock<StatsBlockWithFrames>> = Arc::new(RwLock::default());
        let logs: Arc<RwLock<Vec<String>>> = Arc::new(RwLock::default());
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
        }).await;

        if config.ui_conf.enabled {
            UiThread::init(last_poll.clone(), logs.clone(), conn.clone()).await;
        }

        if !config.offline {
            GameSubmissionClient::init(sge_recv, log_send.clone()).await;
            WebsocketServer::init(last_poll.clone()).await;
        }

        main_task.run().await;
    }

    pub async fn run(&mut self) {
        let mut interval = tokio::time::interval(Duration::from_secs_f32(1. / 3.));
        loop {
            interval.tick().await;
            tokio::select! {
                new_log = self.state.log_recv.recv() => self.handle_log(new_log.unwrap()).await,
            };
        }
    }

    pub async fn handle_log(&mut self, new_log: String) {
        self.state.logs.write().await.push(new_log);
    }
}
