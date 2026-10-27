use std::{
    env,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

use actix::Addr;
use actix_ws::{Message, MessageStream, Session};
use futures::StreamExt;
use tokio::sync::mpsc;

use crate::entities::protocol::{ClientMsg, ServerMsg};
use crate::server::chess_server::{self, ChessServer};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

static NEXT_CONNECTION_ID: AtomicU64 = AtomicU64::new(0);

/// What `ChessServer` sends to a socket
#[derive(Debug)]
pub enum Outbound {
    Text(String),
    /// The user opened a newer socket, this one closes
    Kick,
}

/// Handle `ChessServer` keeps to reach a socket
#[derive(Clone, Debug)]
pub struct Connection {
    /// Tells apart the sockets of a same user
    pub id: u64,
    tx: mpsc::UnboundedSender<Outbound>,
}

impl Connection {
    /// A connection for tests, with the receiving end of its socket
    #[cfg(test)]
    pub fn for_test(id: u64) -> (Self, mpsc::UnboundedReceiver<Outbound>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (Self { id, tx }, rx)
    }

    pub fn send(&self, outbound: Outbound) {
        // Fails only once the socket closed, then there is no one to deliver to
        let _ = self.tx.send(outbound);
    }
}

/// Runs a websocket session until the socket closes, the client stops
/// answering pings, or the user connects again on another socket.
pub async fn run(
    server: Addr<ChessServer>,
    id: String,
    name: String,
    mut session: Session,
    mut msg_stream: MessageStream,
) {
    // Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    // otherwise we drop connection.
    let client_timeout = Duration::from_secs(
        env::var("CLIENT_TIMEOUT")
            .unwrap_or("10".to_string())
            .parse::<u64>()
            .unwrap_or(10),
    );

    let (tx, mut rx) = mpsc::unbounded_channel();
    let connection = Connection {
        id: NEXT_CONNECTION_ID.fetch_add(1, Ordering::Relaxed),
        tx,
    };
    let connection_id = connection.id;

    // register self in chess server
    server.do_send(chess_server::Connect {
        id: id.clone(),
        name,
        connection,
    });

    let mut hb = Instant::now();
    let mut heartbeat = actix_web::rt::time::interval(HEARTBEAT_INTERVAL);

    let close_reason = loop {
        tokio::select! {
            msg = msg_stream.next() => {
                let Some(Ok(msg)) = msg else {
                    break None;
                };

                log::debug!("WEBSOCKET MESSAGE: {msg:?}");
                match msg {
                    Message::Ping(bytes) => {
                        hb = Instant::now();
                        if session.pong(&bytes).await.is_err() {
                            break None;
                        }
                    }
                    Message::Pong(_) => {
                        hb = Instant::now();
                    }
                    Message::Text(text) => {
                        if let Some(reply) = handle_text(&server, &id, &text) {
                            if session.text(reply).await.is_err() {
                                break None;
                            }
                        }
                    }
                    Message::Binary(_) => log::error!("Unexpected binary"),
                    Message::Close(reason) => break reason,
                    Message::Continuation(_) => break None,
                    Message::Nop => (),
                }
            }
            outbound = rx.recv() => match outbound {
                Some(Outbound::Text(text)) => {
                    if session.text(text).await.is_err() {
                        break None;
                    }
                }
                Some(Outbound::Kick) | None => break None,
            },
            _ = heartbeat.tick() => {
                // check client heartbeats
                if hb.elapsed() > client_timeout {
                    log::info!("Websocket Client heartbeat failed, disconnecting!");
                    break None;
                }

                if session.ping(b"").await.is_err() {
                    break None;
                }
            }
        }
    };

    // notify chess server, it ignores this socket if it was replaced
    server.do_send(chess_server::Disconnect { id, connection_id });
    let _ = session.close(close_reason).await;
}

/// Forwards a client message to the chess server. Returns the error to send
/// back when the frame isn't a valid message.
fn handle_text(server: &Addr<ChessServer>, id: &str, text: &str) -> Option<String> {
    match serde_json::from_str::<ClientMsg>(text) {
        Ok(msg) => {
            server.do_send(chess_server::ClientRequest {
                id: id.to_owned(),
                msg,
            });
            None
        }
        Err(err) => {
            log::debug!("Invalid client message {text:?}: {err}");
            let error = ServerMsg::Error {
                message: format!("invalid message: {err}"),
            };
            Some(serde_json::to_string(&error).expect("server messages serialize"))
        }
    }
}
