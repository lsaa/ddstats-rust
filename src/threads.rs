//
// Thread Configs
//

use spin_sleep::{LoopHelper};
use tokio::runtime::Handle;
use tonic::transport::Channel;
use tui::layout::{Constraint, Direction, Layout};
use websocket::{sync::Server, OwnedMessage};

use crate::{
    client::{Client, GameClientState, GameStatus, SubmitGameEvent},
    config::{self, LogoStyle},
    consts::{LOGO_MINI, LOGO_NEW},
    mem::{GameConnection, StatsBlockWithFrames},
    ui::draw_levi,
    Conn,
};
use std::{
    sync::{
        mpsc::{Receiver, Sender},
        Arc, RwLock,
    },
    thread::{self, JoinHandle},
    time::{Instant},
};

/* Game Poll Thread */
pub struct GameClientThread {
    pub join_handle: JoinHandle<()>,
}

impl GameClientThread {
    pub fn create_and_start(
        last_poll: ArcRw<StatsBlockWithFrames>,
        sender: Sender<SubmitGameEvent>,
        log_sender: Sender<String>,
        game_disconnected: Sender<bool>,
        game_conneceted: Sender<bool>,
    ) -> Self {
        let mut client = Client {
            game_connection: GameConnection::dead_connection(),
            game_state: GameClientState::NotConnected,
            last_game_update: Instant::now(),
            compiled_run: None,
            last_game_state: GameStatus::Title,
            submitted_data: false,
            log_sender: log_sender.clone(),
            conn: (game_conneceted, game_disconnected),
            connecting_start: Instant::now(),
            sender,
        };

        let mut loop_helper = LoopHelper::builder()
            .report_interval_s(0.5)
            .build_with_target_rate(36.0);

        let join_handle = thread::spawn(move || loop {
            let _delta = loop_helper.loop_start();
            client.game_loop();

            if let Some(data) = &client.game_connection.last_fetch {
                if let Ok(mut writer) = last_poll.write() {
                    writer.clone_from(data);
                }
            }
            loop_helper.loop_sleep();
        });

        Self { join_handle }
    }
}

pub struct UiThread {}

impl UiThread {
    pub fn create_and_start(
        latest_data: ArcRw<StatsBlockWithFrames>,
        logs: ArcRw<Vec<String>>,
        connected: ArcRw<Conn>,
    ) {
        let mut term = crate::ui::create_term();
        term.clear().expect("Couldn't clear terminal");
        let cfg = config::CONFIG.with(|e| e.clone());
        let mut loop_helper = LoopHelper::builder()
            .report_interval_s(0.5)
            .build_with_target_rate(14.0);

        thread::spawn(move || loop {
            let _delta = loop_helper.loop_start();
            let read_data = latest_data.read().expect("Couldn't read last data");
            let log_list = logs.read().expect("Poisoned logs!").clone();

            term.draw(|f| {
                let mut layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(100)])
                    .split(f.size());

                if !connected.read().expect("AAA").is_ok && cfg.ui_conf.orb_connection_animation {
                    draw_levi(f, layout[0]);
                    return;
                }

                if cfg.ui_conf.logo_style != LogoStyle::Off {
                    let max_w = LOGO_NEW.lines().fold(
                        LOGO_NEW.lines().next().unwrap().chars().count(),
                        |acc, x| {
                            if x.chars().count() > acc {
                                x.chars().count()
                            } else {
                                acc
                            }
                        },
                    );

                    let height = match cfg.ui_conf.logo_style {
                        LogoStyle::Auto => {
                            if layout[0].width as usize >= max_w {
                                LOGO_NEW.lines().count()
                            } else {
                                LOGO_MINI.lines().count()
                            }
                        }
                        LogoStyle::Mini => LOGO_MINI.lines().count(),
                        LogoStyle::Full => LOGO_NEW.lines().count(),
                        LogoStyle::Off => 0,
                    };

                    layout = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Min(height as u16 + 1),
                            Constraint::Percentage(100),
                        ])
                        .split(f.size());

                    crate::ui::draw_logo(f, layout[0]);
                }

                let mut info = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(100)])
                    .horizontal_margin(0)
                    .vertical_margin(0)
                    .split(layout[layout.len() - 1]);

                if !cfg.ui_conf.hide_logs {
                    info = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Min(20), Constraint::Percentage(100)])
                        .horizontal_margin(0)
                        .vertical_margin(0)
                        .split(layout[layout.len() - 1]);

                    crate::ui::draw_logs(f, info[0], &log_list);
                }

                crate::ui::draw_info_table(f, info[info.len() - 1], &read_data);
            })
            .unwrap();
            loop_helper.loop_sleep();
        });
    }
}

use crate::grpc_models;
use crate::grpc_models::game_recorder_client::GameRecorderClient;

pub struct GrpcThread {
    pub submit_recv: Receiver<SubmitGameEvent>,
    pub client: GameRecorderClient<Channel>,
}

impl GrpcThread {
    pub fn create_and_start(submit: Receiver<SubmitGameEvent>, log_sender: Sender<String>) {
        let cfg = config::CONFIG.with(|z| z.clone());
        let handle = Handle::current();
        handle.spawn(async move {
            let mut client = GameRecorderClient::connect(cfg.grpc_host.clone())
                .await
                .expect("GAMES");
            let res = client
                .client_start(grpc_models::ClientStartRequest {
                    version: "0.6.8".to_string(),
                })
                .await
                .expect("GAMING");
            log::info!("MOTD {}", res.get_ref().motd);

            let mut loop_helper = LoopHelper::builder()
                .report_interval_s(0.5)
                .build_with_target_rate(3.);

            loop {
                let _delta = loop_helper.loop_start();
                let maybe = submit.try_recv();
                if maybe.is_ok() && !cfg.offline {
                    log::info!("Got into ClientSubmitReq");
                    let compiled = maybe.unwrap();
                    let g = grpc_models::SubmitGameRequest::from_compiled_run(compiled.0);
                    let res = client.submit_game(g).await;
                    if res.is_ok() {
                        let res = res.as_ref().unwrap();
                        if cfg.auto_clipboard {
                            // cry
                        }

                        log_sender.send(format!("Submitted {}", res.get_ref().game_id)).expect("FUNNY");
                        log::info!("SUBMIT");
                    } else {
                        log::error!("Failed to submit!! {:?}", res);
                    }
                }
                loop_helper.loop_sleep();
            }
        });
    }
}

pub struct WsThread;

impl WsThread {
    pub fn create_and_start(last_poll: ArcRw<StatsBlockWithFrames>) {
        thread::spawn(move || {
            let mut loop_helper = LoopHelper::builder()
                .report_interval_s(0.5)
                .build_with_target_rate(36.);

            let server = Server::bind("127.0.0.1:13666").unwrap();
            for request in server.filter_map(Result::ok) {
                let _delta = loop_helper.loop_start();
                let local_poll = last_poll.clone();
                thread::spawn(move || {
                    if !request.protocols().contains(&"rust-websocket".to_string()) {
                        request.reject().unwrap();
                        return;
                    }
                    let mut client = request.use_protocol("rust-websocket").accept().unwrap();
                    let message = OwnedMessage::Text("Hello".to_string());
                    client.send_message(&message).unwrap();
                    let (mut receiver, mut sender) = client.split().unwrap();
                    for message in receiver.incoming_messages() {
                        let cv = local_poll.read().expect("AA").clone();
                        let serialized = serde_json::to_string(&cv).expect("E");
                        let message = message.unwrap();
                        match message {
                            OwnedMessage::Close(_) => {
                                let message = OwnedMessage::Close(None);
                                sender.send_message(&message).unwrap();
                                return;
                            }
                            OwnedMessage::Ping(ping) => {
                                let message = OwnedMessage::Pong(ping);
                                sender.send_message(&message).unwrap();
                            }
                            _ => sender
                                .send_message(&OwnedMessage::Text(serialized.clone()))
                                .unwrap(),
                        }
                    }
                });
                loop_helper.loop_sleep();
            }
        });
    }
}

type ArcRw<T> = Arc<RwLock<T>>;
