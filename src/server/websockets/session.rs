use std::{
    env,
    time::{Duration, Instant},
};

use actix::prelude::*;
use actix_web_actors::ws;

use crate::server::chess_server::{self, ChessServer};
use crate::utils::sanitize::{sanitize_room_name, sanitize_username};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Debug)]
pub struct WsChessSession {
    /// unique session id
    pub id: String,

    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    pub hb: Instant,

    /// peer name
    pub name: String,

    /// Chat server
    pub addr: Addr<ChessServer>,
}

impl WsChessSession {
    pub fn new(addr: Addr<ChessServer>, id: String, name: String) -> Self {
        Self {
            id,
            hb: Instant::now(),
            name,
            addr,
        }
    }

    /// helper method that sends ping to client every 5 seconds (HEARTBEAT_INTERVAL).
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        let client_timeout = Duration::from_secs(
            env::var("CLIENT_TIMEOUT")
                .unwrap_or("10".to_string())
                .parse::<u64>()
                .unwrap_or(10),
        );

        ctx.run_interval(HEARTBEAT_INTERVAL, move |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > client_timeout {
                // heartbeat timed out
                log::info!("Websocket Client heartbeat failed, disconnecting!");

                // stop actor, `stopping` notifies the chess server
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}

impl Actor for WsChessSession {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start.
    /// We register ws session with ChatServer
    fn started(&mut self, ctx: &mut Self::Context) {
        // we'll start heartbeat process on session start.
        self.hb(ctx);

        // register self in chat server. `AsyncContext::wait` register
        // future within context, but context waits until this future resolves
        // before processing any other events.
        // HttpContext::state() is instance of WsChatSessionState, state is shared
        // across all routes within application
        let addr = ctx.address();
        self.addr.do_send(chess_server::Connect {
            id: self.id.clone(),
            name: self.name.clone(),
            addr,
        })
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        // notify chat server, it ignores this socket if it was replaced
        self.addr.do_send(chess_server::Disconnect {
            id: self.id.clone(),
            addr: ctx.address(),
        });
        Running::Stop
    }
}

/// Handle messages from chat server, we simply send it to peer websocket
impl Handler<chess_server::Message> for WsChessSession {
    type Result = ();

    fn handle(&mut self, msg: chess_server::Message, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl Handler<chess_server::UserSync> for WsChessSession {
    type Result = ();

    fn handle(&mut self, msg: chess_server::UserSync, _: &mut Self::Context) {
        self.name = msg.name;
    }
}

impl Handler<chess_server::Kick> for WsChessSession {
    type Result = ();

    fn handle(&mut self, _: chess_server::Kick, ctx: &mut Self::Context) {
        ctx.stop();
    }
}

/// WebSocket message handler
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsChessSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        log::debug!("WEBSOCKET MESSAGE: {msg:?}");
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                let m = text.trim();
                // we check for /sss type of messages
                if m.starts_with('/') {
                    let (cmd, input) = m.split_once(' ').unwrap_or((m, ""));
                    match cmd {
                        "/join" => {
                            // `/join <room> [<fen>[|<trash>]]`
                            let (room_name, position) = input.split_once(' ').unwrap_or((input, ""));
                            let position = position.trim();
                            let Some(room_name) = sanitize_room_name(room_name) else {
                                ctx.text("!!! room name is required");
                                return;
                            };
                            let (fen, trash) = match position.split_once('|') {
                                Some((fen, trash)) => (Some(fen.to_owned()), Some(trash.to_owned())),
                                None => ((!position.is_empty()).then(|| position.to_owned()), None),
                            };

                            self.addr.do_send(chess_server::Join {
                                id: self.id.clone(),
                                name: room_name,
                                fen,
                                trash,
                            });
                        }
                        "/username" => {
                            if let Some(name) = sanitize_username(input) {
                                self.name = name;

                                self.addr.do_send(chess_server::UserSync {
                                    id: self.id.clone(),
                                    name: self.name.clone(),
                                });
                            } else {
                                ctx.text("!!! name is required");
                            }
                        }
                        "/move" => {
                            let v: Vec<&str> = input.splitn(3, ' ').collect();
                            if v.len() == 3 {
                                let piece = v[0].to_owned();
                                let from = v[1].to_owned();
                                let to = v[2].to_owned();

                                self.addr.do_send(chess_server::Move {
                                    id: self.id.clone(),
                                    piece,
                                    from,
                                    to,
                                });
                            } else {
                                ctx.text("!!! move is required");
                            }
                        }
                        "/reset" => {
                            self.addr.do_send(chess_server::Reset {
                                id: self.id.clone(),
                            });
                        }
                        "/undo" => {
                            self.addr.do_send(chess_server::Undo {
                                id: self.id.clone(),
                            });
                        }
                        "/redo" => {
                            self.addr.do_send(chess_server::Redo {
                                id: self.id.clone(),
                            });
                        }
                        "/options" => {
                            self.addr.do_send(chess_server::Options {
                                id: self.id.clone(),
                                validation: input.contains("validation"),
                                sync: input.contains("sync"),
                            });
                        }
                        _ => ctx.text(format!("!!! unknown command: {m:?}")),
                    }
                } else {
                    let msg = format!("{name}: {m}", name = self.name);
                    // send message to chat server
                    self.addr.do_send(chess_server::ClientMessage {
                        id: self.id.clone(),
                        msg,
                    })
                }
            }
            ws::Message::Binary(_) => log::error!("Unexpected binary"),
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}
