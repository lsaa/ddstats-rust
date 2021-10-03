//
//  websocket_server.rs - Funny Data for Funny Readers
//

use crate::mem::StatsBlockWithFrames;
use futures::SinkExt;
use futures::{stream::SplitSink, StreamExt};
use regex::{Match, Regex};
use ron::ser::{to_string_pretty, PrettyConfig};
use std::convert::Infallible;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;
use tui::style::{Color, Style};
use warp::sse::Event;
use warp::{
    ws::{Message, WebSocket},
    Filter,
};
use std::cell::RefCell;

pub struct WebsocketServer;

type ColorStyles = Arc<RwLock<crate::config::Styles>>;
type Snowflake = Arc<RwLock<u128>>;

thread_local! {
    static LAST_SNOWFLAKE: RefCell<u128> = RefCell::new(0);
}

impl WebsocketServer {
    pub async fn init(poll: PollData, styles: ColorStyles, snowflake: Snowflake) {
        tokio::spawn(async move {
            log::info!("initializing server on port: 13666");

            let health_check = warp::path("health-check").map(|| format!("Server OK"));

            let ws = warp::path::end()
                .and(warp::ws())
                .and(with_poll_data(poll.clone()))
                .and(with_color_edit_styles(styles.clone()))
                .map(|ws: warp::ws::Ws, poll, styles| {
                    log::info!("upgrading connection to websocket");
                    ws.on_upgrade(move |websocket| handle_ws_client(websocket, poll, styles))
                });

            let stream = warp::path("miniblock")
                .and(warp::get())
                .and(with_poll_data(poll.clone()))
                .and(with_snowflake(snowflake.clone()))
                .map(|poll: PollData, flake: Snowflake| {
                    let interval = interval(Duration::from_secs_f32(1. / 36.));
                    let mut is_first = true;
                    let stream = IntervalStream::new(interval);
                    let event_stream = stream.map(move |_instant| {
                        if is_first {
                            is_first = false;
                            return sse_first();
                        }
                        let d = futures::executor::block_on(poll.read()).clone();

                        let mini = MiniBlock::from_stats(
                            &d,
                            futures::executor::block_on(flake.read()).clone()
                        );

                        sse_miniblock(mini, &d)
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

async fn handle_ws_client(websocket: warp::ws::WebSocket, data: PollData, styles: ColorStyles) {
    let (mut sender, mut receiver) = websocket.split();

    while let Some(body) = receiver.next().await {
        let message = match body {
            Ok(msg) => msg,
            Err(e) => {
                log::error!("error reading message on websocket: {}", e);
                break;
            }
        };

        handle_websocket_message(message, &mut sender, data.clone(), styles.clone()).await;
    }

    log::info!("client disconnected");
}

async fn handle_websocket_message(
    message: Message,
    sender: &mut SplitSink<WebSocket, Message>,
    data: PollData,
    styles: ColorStyles,
) {
    let msg = if let Ok(s) = message.to_str() {
        s
    } else {
        log::info!("ping-pong");
        return;
    };

    if msg.eq("gimme") {
        let _ = sender
            .send(Message::text(
                serde_json::to_string(&data.read().await.clone()).unwrap(),
            ))
            .await;
    }

    if msg.starts_with("clr-set") {
        let re =
            Regex::new(r"clr-set\s(\w*)\s(\w*)\s(\d*)\s(\d*)\s(\d*)\s(\w*)\s(\d*)\s(\d*)\s(\d*)")
                .unwrap();
        for cap in re.captures_iter(msg) {
            let mut writer = styles.write().await;

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

            let _ = sender.send(Message::text("Color Set!")).await;
        }
    }

    if msg.starts_with("get-style") {
        let r = styles.read().await.clone();
        let pretty = PrettyConfig::new();
        let s = to_string_pretty(&r, pretty).expect("Serialization failed");
        let sent = sender.send(Message::text(s)).await;
        log::info!("Get Style Request: {:?}", sent);
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

type PollData = Arc<RwLock<StatsBlockWithFrames>>;

fn with_poll_data(c: PollData) -> impl Filter<Extract = (PollData,), Error = Infallible> + Clone {
    warp::any().map(move || c.clone())
}

fn with_snowflake(c: Snowflake) -> impl Filter<Extract = (Snowflake,), Error = Infallible> + Clone {
    warp::any().map(move || c.clone())
}

fn with_color_edit_styles(
    c: ColorStyles,
) -> impl Filter<Extract = (ColorStyles,), Error = Infallible> + Clone {
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
pub struct MiniDto {
    #[serde(rename = "type")]
    pub _type: String,
    pub data: MiniBlock,
    pub extra: Option<StatsBlockWithFrames>
}

impl MiniBlock {
    pub fn from_stats(data: &StatsBlockWithFrames, snowflake: u128) -> Self {
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
            snowflake
        }
    }
}

fn sse_first() -> Result<Event, Infallible> {
    Ok(warp::sse::Event::default().data("{\"type\":\"hello\"}".to_string()))
}

fn sse_miniblock(miniblock: MiniBlock, data: &StatsBlockWithFrames) -> Result<Event, Infallible> {
    LAST_SNOWFLAKE.with(|sn| {
        let mut sn = sn.borrow_mut();
        let mut extra = None;
        if sn.ne(&miniblock.snowflake) {
            *sn = miniblock.snowflake;
            let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() - miniblock.snowflake;
            if t < Duration::from_secs(20).as_millis() {
                extra = Some(data.clone());
            }
        }
        let pain = serde_json::to_string(&MiniDto {
            _type: "miniblock".into(),
            data: miniblock,
            extra
        });
        Ok(warp::sse::Event::default().data(pain.unwrap()))
    })
}
