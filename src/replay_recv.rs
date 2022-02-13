//
//  replay_recv.rs - local replay receiver (Open With...)
//

use std::time::Duration;
use tokio::{net::{TcpListener, TcpStream}, io::{AsyncReadExt, AsyncWriteExt}};
use crate::threads::{AAS, State};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LocalFileReplayMsg {
    pub path: String
}

async fn process_socket(mut socket: TcpStream, state: AAS<State>) {
    let _ = socket.readable().await;
    tokio::time::interval(Duration::from_millis(50)).tick().await;
    let mut txt_buf = [0u8; 2000];
    if let Ok(_read) = socket.read(&mut txt_buf).await {
        if let Ok(read_as_string) = String::from_utf8(txt_buf.to_vec()) {
            let read_as_string = read_as_string.trim_matches(char::from(0)).to_owned();
            let r: Result<LocalFileReplayMsg, serde_json::Error> = serde_json::from_str(&read_as_string);
            if r.is_err() {
                return;
            }
            let r = r.unwrap();
            let _ = state.load().msg_bus.0.send(crate::threads::Message::PlayReplayLocalFile(r.path));
        }
    }
}

pub struct LocalReplayReceiver;

impl LocalReplayReceiver {
    pub async fn init(state: AAS<State>) {
        tokio::spawn(async move {
            let listener = TcpListener::bind("127.0.0.1:18639").await.unwrap();

            loop {
                match listener.accept().await {
                    Ok((socket, _)) => process_socket(socket, state.clone()).await,
                    Err(_e) => {}
                }
            }
        });
    }
}

pub async fn send_to_current_instance(path: String) {
    let j = serde_json::json!({
        "path": path
    }).to_string();

    if let Ok(mut connection) = TcpStream::connect("127.0.0.1:18639").await {
        let _ = connection.writable().await;
        let _ = connection.try_write(j.as_bytes());
        let _ = connection.flush();
    }
}
