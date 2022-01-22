//
//  websocket_server.rs - Funny Data for Funny Readers
//

use anyhow::{Result, bail};
use ddcore_rs::models::{StatsBlockWithFrames, StatsDataBlock, StatsFrame};
use futures::SinkExt;
use futures::{stream::SplitSink, StreamExt};
use hyper::client::HttpConnector;
use hyper::{Body, Method, Request};
use regex::{Match, Regex};
use ron::ser::{to_string_pretty, PrettyConfig};
use serde::Serialize;
use std::convert::Infallible;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;
use tui::style::{Color, Style};
use warp::sse::Event;
use warp::{
    ws::{Message, WebSocket},
    Filter,
};

use crate::client::ConnectionState;
use crate::threads::{AAS, State};

#[derive(Debug, Serialize, Clone)]
pub struct WsBroadcast {
    #[serde(rename = "type")]
    pub _type: String,
    pub data: String,
}
pub struct WebsocketServer;

static LAST_SNOWFLAKE: AtomicU64 = AtomicU64::new(0);

impl WebsocketServer {
    pub async fn init(state: AAS<State>) {
        tokio::spawn(async move {
            log::info!("initializing server on port: 13666");

            let health_check = warp::path("health-check").map(|| format!("Server OK"));

            let ws = warp::path::end()
                .and(warp::ws())
                .and(with_state_data(state.clone()))
                .map(|ws: warp::ws::Ws, state| {
                    log::info!("upgrading connection to websocket");
                    ws.on_upgrade(move |websocket| handle_ws_client(websocket, state))
                });

            let stream = warp::path("miniblock")
                .and(warp::get())
                .and(with_state_data(state.clone()))
                .map(|state: AAS<State>| {
                    let interval = interval(Duration::from_secs_f32(1. / 36.));
                    let mut is_first = true;
                    let stream = IntervalStream::new(interval);
                    let event_stream = stream.map(move |_instant| {
                        let state = state.load();

                        if is_first {
                            is_first = false;
                            return sse_first();
                        }

                        let mini = MiniBlock::from_stats(
                            state.last_poll.clone(),
                            state.snowflake.clone()
                        );

                        sse_miniblock(mini, state.last_poll.clone())
                    });
                    warp::sse::reply(event_stream)
                });

            let routes = health_check
                .or(ws)
                .or(stream)
                .with(warp::cors().allow_any_origin());

            warp::serve(routes)
                .run(SocketAddr::new(
                    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                    13666,
                ))
                .await;
            log::info!("server is running");
        });
    }
}

async fn handle_ws_client(
    websocket: warp::ws::WebSocket, 
    state: AAS<State>
) {
    let (mut sender, mut receiver) = websocket.split();
    let mut msg_bus = state.load().msg_bus.0.subscribe();

    loop {
        tokio::select! {
            msg = msg_bus.recv() => match msg {
                Ok(crate::threads::Message::WebSocketMessage(data)) => {
                    let t = serde_json::to_string(&data).unwrap();
                    let _ = sender.send(Message::text(t)).await;
                },
                _ => {},
            },
            body = receiver.next() => {
                let message = match body {
                    Some(Ok(msg)) => msg,
                    Some(Err(e)) => {
                        log::error!("error reading message on websocket: {}", e);
                        break;
                    },
                    None => { break; }
                };
        
                handle_websocket_message(message, &mut sender, state.clone()).await;
            }
        }
    }

    log::info!("websocket client disconnected");
}

async fn handle_websocket_message(
    message: Message,
    sender: &mut SplitSink<WebSocket, Message>,
    data: AAS<State>,
) {
    let msg = if let Ok(s) = message.to_str() {
        s
    } else {
        log::info!("ping-pong");
        return;
    };
    let state = data.load();

    if msg.eq("gimme") {
        let mut v = StatsDto::from_sbwf(state.last_poll.clone());
        let s = (*state.conn).clone();
        v.additional_info.connection_state = Some(s);
        let t = format!("{{\"type\": \"fullblock\", \"data\": {} }}", serde_json::to_string(&v).unwrap());
        let _ = sender.send(Message::text(t)).await;
    }

    if msg.eq("config") {
        let t = serde_json::to_string(crate::config::cfg().as_ref()).unwrap();
        let t = format!("{{\"type\": \"config\", \"data\": {} }}", t);
        let _ = sender.send(Message::text(t)).await;
    }

    if msg.starts_with("ddcl_replay") {
        let id = i32::from_str_radix(msg.split(" ").nth(1).unwrap(), 10).unwrap();
        let cfg = crate::config::cfg();
        if (*state.conn).eq(&ConnectionState::NotConnected) && cfg.open_game_on_replay_request {
            log::info!("Opened DD: {:?}", ddcore_rs::memory::start_dd());
        }
        let replay_sender = state.msg_bus.0.clone();
        tokio::spawn(async move {
            if let Ok(replay) = ddcore_rs::ddinfo::get_replay_by_id(id).await {
                let _ = replay_sender.send(crate::threads::Message::Replay(Arc::new(replay)));
            } else {
                log::warn!("Failed to load DDCL replay {}", id);
            }
        });
        let t = format!("{{\"type\": \"ddcl_replay_ok\", \"data\": {{ \"replay_id\": {} }} }}", id);
        let _ = sender.send(Message::text(t)).await;
    }

    if msg.starts_with("replay_link") {
        let link = format!("{}", msg.clone().split(" ").nth(1).unwrap());
        let lc = link.clone();
        let cfg = crate::config::cfg();
        if (*state.conn).eq(&ConnectionState::NotConnected) && cfg.open_game_on_replay_request {
            log::info!("Opened DD: {:?}", ddcore_rs::memory::start_dd());
        }
        let replay_sender = state.msg_bus.0.clone();
        tokio::spawn(async move {
            if let Ok(replay) = get_replay_link(&link).await {
                let _ = replay_sender.send(crate::threads::Message::Replay(Arc::new(replay)));
            } else {
                log::warn!("Failed to load replay {}", link);
            }
        });
        let t = format!("{{\"type\": \"replay_link_ok\", \"data\": {{ \"replay_link\": \"{}\" }} }}", lc);
        let _ = sender.send(Message::text(t)).await;
    }

    if msg.starts_with("clr-set") {
        let re =
            Regex::new(r"clr-set\s(\w*)\s(\w*)\s(\d*)\s(\d*)\s(\d*)\s(\w*)\s(\d*)\s(\d*)\s(\d*)")
                .unwrap();
        for cap in re.captures_iter(msg) {
            let mut writer = (*state.color_edit).clone();

            let (
                field,
                color_type1,
                color1r,
                color1g,
                color1b, // BG
                color_type2,
                color2r,
                color2g,
                color2b, // FG
            ) = (
                cap.get(1).unwrap(),
                cap.get(2).unwrap(),
                cap.get(3).unwrap(),
                cap.get(4).unwrap(),
                cap.get(5).unwrap(),
                cap.get(6).unwrap(),
                cap.get(7).unwrap(),
                cap.get(8).unwrap(),
                cap.get(9).unwrap(),
            );

            let bg = color_from_match(color_type1, color1r, color1g, color1b);
            let fg = color_from_match(color_type2, color2r, color2g, color2b);

            match field.as_str() {
                "logo" => writer.logo = Style::default().fg(fg).bg(bg),
                "logs" => writer.logs = Style::default().fg(fg).bg(bg),
                "log_text" => writer.log_text = Style::default().fg(fg).bg(bg),
                "most_recent_log" => writer.most_recent_log = Style::default().fg(fg).bg(bg),
                "game_data" => writer.game_data = Style::default().fg(fg).bg(bg),
                "split_name" => writer.split_name = Style::default().fg(fg).bg(bg),
                "split_value" => writer.split_value = Style::default().fg(fg).bg(bg),
                "split_diff_pos" => writer.split_diff_pos = Style::default().fg(fg).bg(bg),
                "split_diff_neg" => writer.split_diff_neg = Style::default().fg(fg).bg(bg),
                _ => {}
            };

            let color_sender = state.msg_bus.0.clone();   
            let _  = color_sender.send(crate::threads::Message::NewColorEdit(Arc::new(writer)));

            let t = format!("{{\"type\": \"color_set_ok\", \"data\": null }}");
            let _ = sender.send(Message::text(t)).await;
        }
    }

    if msg.starts_with("get-style") {
        let r = (*state.color_edit).clone();
        let pretty = PrettyConfig::new();
        let s = to_string_pretty(&r, pretty).expect("Serialization failed");
        let t = format!("{{\"type\": \"get_style\", \"data\": {} }}", s);
        let _sent = sender.send(Message::text(t)).await;
    }
}

struct ColorProxy(pub Color);

impl FromStr for ColorProxy {
    type Err = ();

    fn from_str(input: &str) -> Result<ColorProxy, Self::Err> {
        match input {
            "Reset" => Ok(ColorProxy(Color::Reset)),
            "Black" => Ok(ColorProxy(Color::Black)),
            "Red" => Ok(ColorProxy(Color::Red)),
            "Green" => Ok(ColorProxy(Color::Green)),
            "Yellow" => Ok(ColorProxy(Color::Yellow)),
            "Blue" => Ok(ColorProxy(Color::Blue)),
            "Magenta" => Ok(ColorProxy(Color::Magenta)),
            "Cyan" => Ok(ColorProxy(Color::Cyan)),
            "Gray" => Ok(ColorProxy(Color::Gray)),
            "DarkGray" => Ok(ColorProxy(Color::DarkGray)),
            "LightRed" => Ok(ColorProxy(Color::LightRed)),
            "LightGreen" => Ok(ColorProxy(Color::LightGreen)),
            "LightYellow" => Ok(ColorProxy(Color::LightYellow)),
            "LightBlue" => Ok(ColorProxy(Color::LightBlue)),
            "LightMagenta" => Ok(ColorProxy(Color::LightMagenta)),
            "LightCyan" => Ok(ColorProxy(Color::LightCyan)),
            "White" | _ => Ok(ColorProxy(Color::White)),
        }
    }
}

fn color_from_match(enum_type: Match, red: Match, green: Match, blue: Match) -> Color {
    match enum_type.as_str() {
        "Rgb" => Color::Rgb(
            u8::from_str_radix(red.as_str(), 10).unwrap(),
            u8::from_str_radix(green.as_str(), 10).unwrap(),
            u8::from_str_radix(blue.as_str(), 10).unwrap(),
        ),
        "Indexed" => Color::Indexed(u8::from_str_radix(red.as_str(), 10).unwrap()),
        v => ColorProxy::from_str(v).unwrap().0,
    }
}


fn with_state_data(c: AAS<State>) -> impl Filter<Extract = (AAS<State>,), Error = Infallible> + Clone {
    warp::any().map(move || c.clone())
}

#[derive(serde::Serialize)]
pub struct MiniBlock {
    pub time: f32,
    pub daggers_fired: i32,
    pub daggers_hit: i32,
    pub enemies_alive: i32,
    pub gems_collected: i32,
    pub gems_despawned: i32,
    pub gems_eaten: i32,
    pub gems_total: i32,
    pub homing: i32,
    pub kills: i32,
    pub status: i32,
    pub snowflake: u128
}

#[derive(serde::Serialize)]
pub struct FullDto {
    #[serde(rename = "type")]
    pub _type: String,
    pub data: StatsBlockWithFrames,
}

#[derive(serde::Serialize)]
pub struct StatsDto {
    pub block: StatsDataBlock,
    pub frames: Vec<StatsFrame>,
    pub additional_info: AdditionalInfo,
}

#[derive(serde::Serialize)]
pub struct AdditionalInfo {
    pub frame_count: usize,
    pub connection_state: Option<ConnectionState>,
}

impl StatsDto {
    pub fn from_sbwf(data: Arc<StatsBlockWithFrames>) -> Self {
        let s = data.frames.len();
        Self {
            block: data.block.clone(),
            frames: data.frames.clone(),
            additional_info: AdditionalInfo {
                frame_count: s,
                connection_state: None
            }
        }
    }
}

#[derive(serde::Serialize)]
pub struct MiniDto {
    #[serde(rename = "type")]
    pub _type: String,
    pub data: MiniBlock,
    pub extra: Option<StatsDto>
}

impl MiniBlock {
    pub fn from_stats(data: Arc<StatsBlockWithFrames>, snowflake: Arc<u128>) -> Self {
        Self {
            time: data.block.time,
            daggers_fired: data.block.daggers_fired,
            daggers_hit: data.block.daggers_hit,
            enemies_alive: data.block.enemies_alive,
            gems_collected: data.block.gems_collected,
            gems_despawned: data.block.gems_despawned,
            gems_eaten: data.block.gems_eaten,
            gems_total: data.block.gems_total,
            homing: data.block.homing,
            kills: data.block.kills,
            status: data.block.status,
            snowflake: *snowflake
        }
    }
}

fn sse_first() -> Result<Event, Infallible> {
    Ok(warp::sse::Event::default().data("{\"type\":\"hello\"}".to_string()))
}

fn _sse_empty() -> Result<Event, Infallible> {
    Ok(warp::sse::Event::default().data("{\"type\":\"empty\"}".to_string()))
}

fn sse_miniblock(miniblock: MiniBlock, data: Arc<StatsBlockWithFrames>) -> Result<Event, Infallible> {
    let sn = &LAST_SNOWFLAKE;
    let v = sn.load(std::sync::atomic::Ordering::Relaxed);
    let mut extra = None;
    if v != miniblock.snowflake as u64 {
        sn.store(miniblock.snowflake as u64, std::sync::atomic::Ordering::Release);
        let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() - miniblock.snowflake;
        if t < Duration::from_secs(20).as_millis() {
            let mut extrae = (*data).clone();
            if data.frames.len() > 0 {
                extrae.frames = vec![data.frames[0]];
            } else {
                extrae.frames = vec![];
            }
            extra = Some(StatsDto::from_sbwf(Arc::new(extrae)));
        }
    }
    let pain = serde_json::to_string(&MiniDto {
        _type: "miniblock".into(),
        data: miniblock,
        extra
    });
    Ok(warp::sse::Event::default().data(pain.unwrap()))
}

pub async fn get_replay_link(link: &str) -> Result<Vec<u8>> {
    let mut tls_connector_builder = native_tls::TlsConnector::builder();
    tls_connector_builder.danger_accept_invalid_hostnames(true);
    tls_connector_builder.danger_accept_invalid_certs(true);
    let tls_connector = tls_connector_builder.build().unwrap();
    let mut http = HttpConnector::new();
    http.enforce_http(false);
    let https = hyper_tls::HttpsConnector::from((http, tls_connector.into()));
    let client = hyper::Client::builder().build(https);
    let uri = format!("{}", link);
    let req = Request::builder()
        .method(Method::GET)
        .uri(uri)
        .body(Body::empty())
        .unwrap();
    let mut res = client.request(req).await?;
    let mut body = Vec::new();
    while let Some(chunk) = res.body_mut().next().await {
        body.extend_from_slice(&chunk?);
    }
    if res.status() != 200 {
        unsafe { bail!(String::from_utf8_unchecked(body)); }
    }
    Ok(body)
}
