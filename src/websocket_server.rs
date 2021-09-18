//
//  websocket_server.rs - Funny Data for Funny Readers
//

use crate::mem::StatsBlockWithFrames;
use futures::SinkExt;
use futures::{stream::SplitSink, StreamExt};
use std::convert::Infallible;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::{Instant, interval};
use tokio_stream::wrappers::IntervalStream;
use warp::sse::Event;
use warp::{
    ws::{Message, WebSocket},
    Filter,
};

pub struct WebsocketServer;

impl WebsocketServer {
    pub async fn init(poll: PollData) {
        tokio::spawn(async move {
            log::info!("initializing server on port: 13666");

            let health_check = warp::path("health-check").map(|| format!("Server OK"));

            let ws = warp::path::end()
                .and(warp::ws())
                .and(with_poll_data(poll.clone()))
                .map(|ws: warp::ws::Ws, poll| {
                    log::info!("upgrading connection to websocket");
                    ws.on_upgrade(move |websocket| handle_ws_client(websocket, poll))
                });

            let stream = warp::path("miniblock")
                .and(warp::get())
                .and(with_poll_data(poll.clone()))
                .map(|poll: PollData| {
                    let interval = interval(Duration::from_secs_f32(1. / 36.));
                    let mut is_first = true;
                    let stream = IntervalStream::new(interval);
                    let event_stream = stream.map(move |_instant| {
                        if is_first {
                            is_first = false;
                            return sse_first();
                        }

                        let mini = MiniBlock::from_stats(
                            &futures::executor::block_on(poll.read()).clone(),
                        );

                        sse_miniblock(mini)
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

async fn handle_ws_client(websocket: warp::ws::WebSocket, data: PollData) {
    let (mut sender, mut receiver) = websocket.split();

    while let Some(body) = receiver.next().await {
        let message = match body {
            Ok(msg) => msg,
            Err(e) => {
                log::error!("error reading message on websocket: {}", e);
                break;
            }
        };

        handle_websocket_message(message, &mut sender, data.clone()).await;
    }

    log::info!("client disconnected");
}

async fn handle_websocket_message(
    message: Message,
    sender: &mut SplitSink<WebSocket, Message>,
    data: PollData,
) {
    let _msg = if let Ok(s) = message.to_str() {
        s
    } else {
        log::info!("ping-pong");
        return;
    };

    sender
        .send(Message::text(
            serde_json::to_string(&data.read().await.clone()).unwrap(),
        ))
        .await
        .unwrap();
}

type PollData = Arc<RwLock<StatsBlockWithFrames>>;

fn with_poll_data(c: PollData) -> impl Filter<Extract = (PollData,), Error = Infallible> + Clone {
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
}

#[derive(serde::Serialize)]
pub struct FullDto {
    #[serde(rename = "type")]
    pub _type: String,
    pub data: StatsBlockWithFrames
}

#[derive(serde::Serialize)]
pub struct MiniDto {
    #[serde(rename = "type")]
    pub _type: String,
    pub data: MiniBlock
}

impl MiniBlock {
    pub fn from_stats(data: &StatsBlockWithFrames) -> Self {
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
        }
    }
}
fn sse_first() -> Result<Event, Infallible> {
    Ok(warp::sse::Event::default().data("{\"type\":\"hello\"}".to_string()))
}

fn sse_full(miniblock: StatsBlockWithFrames) -> Result<Event, Infallible> {
    let pain = serde_json::to_string(&FullDto { _type: "full".into(), data: miniblock });
    log::info!("{:?}", pain);
    Ok(warp::sse::Event::default().data(pain.unwrap()))
}

fn sse_miniblock(miniblock: MiniBlock) -> Result<Event, Infallible> {
    let pain = serde_json::to_string(&MiniDto { _type: "miniblock".into(), data: miniblock });
    log::info!("{:?}", pain);
    Ok(warp::sse::Event::default().data(pain.unwrap()))
}
