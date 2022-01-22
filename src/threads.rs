//  The best version.
//  threads.rs - Management of threads 
//  Rewrite Counter: 3 x (I HATE WINDOWS)

use crate::{client::{ConnectionState, GamePollClient, SubmitGameEvent}, config::cfg, grpc_client::GameSubmissionClient, socketio_client::LiveGameClient, ui::UiThread, websocket_server::{WebsocketServer, WsBroadcast}, discord::RichPresenceClient};
use std::{sync::Arc, time::UNIX_EPOCH};
use arc_swap::ArcSwap;
use ddcore_rs::models::StatsBlockWithFrames;
use crate::socketio_client::SubmitSioEvent;

pub type AAS<T> = Arc<ArcSwap<T>>;

#[derive(Clone)]
pub enum Message {
    Log(String),
    SubmitGame(Arc<SubmitGameEvent>),
    NewGameData(Arc<StatsBlockWithFrames>),
    NewSnowflake(Arc<u128>),
    NewColorEdit(Arc<crate::config::Styles>),
    NewConnectionState(Arc<ConnectionState>),
    WebSocketMessage(WsBroadcast),
    SocketIoMessage(SubmitSioEvent),
    UploadReplayBuffer,
    Replay(Arc<Vec<u8>>),
    Exit,
}

#[derive(Clone)]
pub struct State {
    pub conn: Arc<ConnectionState>,
    pub last_poll: Arc<StatsBlockWithFrames>,
    pub snowflake: Arc<u128>,
    pub color_edit: Arc<crate::config::Styles>,
    pub msg_bus: Arc<(tokio::sync::broadcast::Sender<Message>, tokio::sync::broadcast::Receiver<Message>)>
}

#[rustfmt::skip]
pub async fn init() {
    let config = cfg();
    let msg_bus = Arc::new(tokio::sync::broadcast::channel(32));

    let state = Arc::new(ArcSwap::from_pointee(State {
        conn: Arc::default(),
        last_poll: Arc::default(),
        color_edit: Arc::default(),
        snowflake: Arc::new(std::time::SystemTime::now().duration_since(UNIX_EPOCH).expect("Couldn't create snowflake").as_millis()),
        msg_bus
    }));

    GamePollClient::init(state.clone()).await;

    if config.ui_conf.enabled {
        UiThread::init(state.clone()).await;
    }

    if !config.offline {
        log::info!("ONLINE MODE!");
        GameSubmissionClient::init(state.clone()).await;
        WebsocketServer::init(state.clone()).await;
        LiveGameClient::init(state.clone()).await;
        RichPresenceClient::init(state.clone()).await;
    }

    let mut bus_recv = state.load().msg_bus.0.subscribe();
    loop {
        tokio::select! {
            msg = bus_recv.recv() => match msg {
                Ok(Message::NewGameData(data)) => {
                    let mut old = (*state.load_full()).clone();
                    old.last_poll = data;
                    state.swap(Arc::new(old));
                },
                Ok(Message::NewSnowflake(data)) => {
                    let mut old = (*state.load_full()).clone();
                    old.snowflake = data;
                    state.swap(Arc::new(old));
                },
                Ok(Message::NewColorEdit(data)) => {
                    let mut old = (*state.load_full()).clone();
                    old.color_edit = data;
                    state.swap(Arc::new(old));
                },
                Ok(Message::NewConnectionState(data)) => {
                    let mut old = (*state.load_full()).clone();
                    old.conn = data;
                    state.swap(Arc::new(old));
                },
                Ok(Message::Exit) => { break; },
                _ => {}
            },
        };
    }
}
