use std::{
    env,
    time::{Duration, Instant},
};

use actix::prelude::*;
use actix_web_actors::ws;

use crate::server::chess_server::{self, ChessServer};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Debug)]
pub struct WsChessSession {
    /// unique session id
    pub id: usize,

    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    pub hb: Instant,

    /// joined room
    pub room_name: String,

    /// peer name
    pub name: Option<String>,

    /// Chat server
    pub addr: Addr<ChessServer>,

    /// Is authenticated
    pub authenticated_at: Option<Instant>,
}

impl WsChessSession {
    pub fn new(addr: Addr<ChessServer>) -> Self {
        Self {
            id: 0,
            hb: Instant::now(),
            room_name: "main".to_owned(),
            name: None,
            addr,
            authenticated_at: None,
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

                // notify chat server
                act.addr.do_send(chess_server::Disconnect { id: act.id });

                // stop actor
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
        self.addr
            .send(chess_server::Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok((id, _fen)) => act.id = id,
                    // something is wrong with chat server
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // notify chat server
        self.addr.do_send(chess_server::Disconnect { id: self.id });
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
                        // "/list" => {
                        //     // Send ListRooms message to chat server and wait for
                        //     // response
                        //     log::info!("List matches");
                        //     self.addr
                        //         .send(server::chess_server::ListMatches)
                        //         .into_actor(self)
                        //         .then(|res, _, ctx| {
                        //             match res {
                        //                 Ok(matches) => {
                        //                     for match_name in matches {
                        //                         ctx.text(match_name);
                        //                     }
                        //                 }
                        //                 _ => log::error!("Something is wrong"),
                        //             }
                        //             fut::ready(())
                        //         })
                        //         .wait(ctx)
                        //     // .wait(ctx) pauses all events in context,
                        //     // so actor wont receive any new messages until it get list
                        //     // of rooms back
                        // }
                        "/join" => {
                            let v: Vec<&str> = input.splitn(2, ' ').collect();
                            if v.len() >= 1 && v.len() <= 2 {
                                self.room_name = v[0].to_owned();
                                let params: Option<(Option<String>, Option<String>)> =
                                    v.get(1).map(|s| match s.to_owned().split_once("|") {
                                        Some((fen, trash)) => {
                                            (Some(fen.to_owned()), Some(trash.to_owned()))
                                        }
                                        None => (Some(s.to_owned().to_owned()), None),
                                    });
                                self.addr.do_send(chess_server::Join {
                                    id: self.id,
                                    name: self.room_name.clone(),
                                    fen: params.clone().unwrap_or((None, None)).0,
                                    trash: params.unwrap_or((None, None)).1,
                                });

                                ctx.text("joined");
                            } else {
                                ctx.text("!!! room name is required");
                            }
                        }
                        "/name" => {
                            if input != "" {
                                self.name = Some(input.to_owned());
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
                                    id: self.id,
                                    room_name: self.room_name.clone(),
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
                                id: self.id,
                                room_name: self.room_name.clone(),
                            });
                        }
                        _ => ctx.text(format!("!!! unknown command: {m:?}")),
                    }
                } else {
                    let msg = if let Some(ref name) = self.name {
                        format!("{name}: {m}")
                    } else {
                        m.to_owned()
                    };
                    // send message to chat server
                    self.addr.do_send(chess_server::ClientMessage {
                        id: self.id,
                        msg,
                        room_name: self.room_name.clone(),
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
