use std::{
    env,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

use actix::Addr;
use actix_ws::{Message, MessageStream, Session};
use futures::StreamExt;
use tokio::sync::mpsc;

use crate::server::chess_server::{self, ChessServer};
use crate::utils::sanitize::{sanitize_room_name, sanitize_username};

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
                        if let Some(reply) = handle_text(&server, &id, text.trim()) {
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

/// Forwards a client line to the chess server. Returns the error to send back
/// when the command is malformed.
fn handle_text(server: &Addr<ChessServer>, id: &str, text: &str) -> Option<String> {
    let id = id.to_owned();

    // we check for /sss type of messages
    if !text.starts_with('/') {
        // send message to chess server
        server.do_send(chess_server::ClientMessage {
            id,
            msg: text.to_owned(),
        });
        return None;
    }

    let (cmd, input) = text.split_once(' ').unwrap_or((text, ""));
    match cmd {
        "/join" => {
            // `/join <room> [<fen>[|<trash>]]`
            let (room_name, position) = input.split_once(' ').unwrap_or((input, ""));
            let position = position.trim();
            let Some(room_name) = sanitize_room_name(room_name) else {
                return Some("!!! room name is required".to_string());
            };
            let (fen, trash) = match position.split_once('|') {
                Some((fen, trash)) => (Some(fen.to_owned()), Some(trash.to_owned())),
                None => ((!position.is_empty()).then(|| position.to_owned()), None),
            };

            server.do_send(chess_server::Join {
                id,
                name: room_name,
                fen,
                trash,
            });
        }
        "/username" => {
            let Some(name) = sanitize_username(input) else {
                return Some("!!! name is required".to_string());
            };

            server.do_send(chess_server::UserSync { id, name });
        }
        "/move" => {
            let v: Vec<&str> = input.splitn(3, ' ').collect();
            let [piece, from, to] = v[..] else {
                return Some("!!! move is required".to_string());
            };

            server.do_send(chess_server::Move {
                id,
                piece: piece.to_owned(),
                from: from.to_owned(),
                to: to.to_owned(),
            });
        }
        "/reset" => server.do_send(chess_server::Reset { id }),
        "/undo" => server.do_send(chess_server::Undo { id }),
        "/redo" => server.do_send(chess_server::Redo { id }),
        "/options" => server.do_send(chess_server::Options {
            id,
            validation: input.contains("validation"),
            sync: input.contains("sync"),
        }),
        _ => return Some(format!("!!! unknown command: {text:?}")),
    }

    None
}
