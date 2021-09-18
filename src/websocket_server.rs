//
//  websocket_server.rs - Funny Data for Funny Readers
//

use futures::SinkExt;
use futures::{stream::SplitSink, StreamExt};
use std::convert::Infallible;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::{
    ws::{Message, WebSocket},
    Filter,
};

use crate::mem::StatsBlockWithFrames;

pub struct WebsocketServer;

impl WebsocketServer {
    pub async fn init(poll: PollData) {
        tokio::spawn(async move {
            log::info!("initializing server on port: 13666");

            let health_check = warp::path("health-check").map(|| format!("Server OK"));

            let ws = warp::path::end()
                .and(warp::ws())
                .and(with_poll_data(poll))
                .map(|ws: warp::ws::Ws, poll| {
                    log::info!("upgrading connection to websocket");
                    ws.on_upgrade(move |websocket| handle_ws_client(websocket, poll))
                });

            let routes = health_check.or(ws).with(warp::cors().allow_any_origin());

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
