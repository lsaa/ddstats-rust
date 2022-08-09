//  The best version.
//  threads.rs - Management of threads 
//  Rewrite Counter: 3 x (I HATE WINDOWS)

use crate::{client::{ConnectionState, GamePollClient, SubmitGameEvent}, grpc_client::GameSubmissionClient, socketio_client::LiveGameClient, ui::UiThread, websocket_server::{WebsocketServer, WsBroadcast}, discord::RichPresenceClient, replay_recv::LocalReplayReceiver};
use std::{sync::Arc, time::{UNIX_EPOCH, Duration}, net::TcpListener};
use arc_swap::ArcSwap;
use clap::Arg;
use ddcore_rs::models::StatsBlockWithFrames;
use crate::socketio_client::SubmitSioEvent;

pub type AAS<T> = Arc<ArcSwap<T>>;

#[derive(Clone)]
pub enum Message {
    Log(String),
    SubmitGame(Arc<SubmitGameEvent>),
    NewGameData(Arc<StatsBlockWithFrames>),
    NewSnowflake(Arc<u128>),
    NewConnectionState(Arc<ConnectionState>),
    WebSocketMessage(WsBroadcast),
    SocketIoMessage(SubmitSioEvent),
    UploadReplayBuffer,
    UploadReplayData(Arc<Vec<u8>>, bool),
    PlayReplayLocalFile(String),
    Replay(Arc<Vec<u8>>),
    ShowWindow,
    HideWindow,
    SaveCfg,
    Exit,
}

#[derive(Clone)]
pub struct State {
    pub conn: Arc<ConnectionState>,
    pub last_poll: Arc<StatsBlockWithFrames>,
    pub snowflake: Arc<u128>,
    pub msg_bus: Arc<(tokio::sync::broadcast::Sender<Message>, tokio::sync::broadcast::Receiver<Message>)>
}

#[rustfmt::skip]
pub async fn init() {
    let msg_bus = Arc::new(tokio::sync::broadcast::channel(512 * 2));
    let cfg = crate::config::cfg();

    let state = Arc::new(ArcSwap::from_pointee(State {
        conn: Arc::default(),
        last_poll: Arc::default(),
        snowflake: Arc::new(std::time::SystemTime::now().duration_since(UNIX_EPOCH).expect("Couldn't create snowflake").as_millis()),
        msg_bus
    }));

    GamePollClient::init(state.clone()).await;
    UiThread::init(state.clone()).await;
    #[cfg(target_os = "windows")] crate::tray::TrayIcon::init(state.clone()).await;
    #[cfg(target_os = "windows")] let _ = winconsole::console::set_title("ddstats-rust");

    let app = clap::App::new("ddstats-rust")
        .bin_name("ddstats-rust")
        .arg(Arg::new("replay")
            .takes_value(true)
            .value_name("FILE")
            .required(false)
            .help("Opens replay with ddstats-rust"));

    let mut repl = None;
    if let Some(replay) = app.get_matches().value_of("replay") {
        if !port_is_available(18639) {
            crate::replay_recv::send_to_current_instance(replay.to_owned()).await;
            return;
        } else {
            repl = Some(replay.to_owned());
        }
    }

    if !port_is_available(18639) {
        log::error!("local replay port already bound, ddstats-rust is probably already open.");
        return;
    }

    LocalReplayReceiver::init(state.clone()).await;

    if !cfg.offline {
        log::info!("ONLINE MODE!");
        GameSubmissionClient::init(state.clone()).await;
        WebsocketServer::init(state.clone()).await;
        LiveGameClient::init(state.clone()).await;
        RichPresenceClient::init(state.clone()).await;
    }

    let mut bus_recv = state.load().msg_bus.0.subscribe();
    
    if let Some(repl) = repl {
        let _ = state.load().msg_bus.0.send(Message::PlayReplayLocalFile(repl));
    }

    let exit_recv = state.load().msg_bus.0.clone();
    std::thread::spawn(move || {
        ctrlc::set_handler(move || {
            log::info!("AAA");
            log::info!("AAA");
            log::info!("AAA");
            log::info!("AAA");
            log::info!("AAA");
            log::info!("AAA");
            log::info!("AAA");
            log::info!("AAA");
            log::info!("AAA");
            let _ = exit_recv.send(Message::Exit);
            std::thread::sleep(Duration::from_secs(3));
        }).expect("Error setting Ctrl-C handler");
    });

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
                Ok(Message::NewConnectionState(data)) => {
                    let mut old = (*state.load_full()).clone();
                    old.conn = data;
                    state.swap(Arc::new(old));
                },
                Ok(Message::SaveCfg) => {
                    log::info!("SAVING CFG: {:?}", crate::config::try_save_with_backup());
                },
                Ok(Message::Exit) => { 
                    log::info!("SAVING CFG: {:?}", crate::config::try_save_with_backup());
                    log::info!("EXIT"); 
                    break; 
                },
                _ => {}
            },
        };
    }
}

pub fn port_is_available(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}
