//! `ChessServer` is an actor. It maintains list of connection client session.
//! And manages available rooms. Peers send messages to other peers in same
//! room through `ChessServer`.

use std::{
    collections::{HashMap, HashSet},
    env,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};

use crate::entities::{chess_board::ChessBoard, position::Position};

/// Chat server sends this messages to session
#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

/// Message for chat server communications

/// New chat session is created
pub struct Connect {
    pub addr: Recipient<Message>,
}

impl actix::Message for Connect {
    type Result = (usize, String);
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

/// Send message to specific room
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    /// Id of the client session
    pub id: usize,
    /// Peer message
    pub msg: String,
    /// Room name
    pub room_name: String,
}

/// List of available rooms
// pub struct ListMatches;
//
// impl actix::Message for ListMatches {
//     type Result = Vec<String>;
// }

/// Join room, if room does not exists create new one.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    /// Client ID
    pub id: usize,

    /// Room name
    pub name: String,

    /// Fen
    pub fen: Option<String>,

    /// Trash
    pub trash: Option<String>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Move {
    pub id: usize,
    pub room_name: String,
    pub piece: String,
    pub from: String,
    pub to: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Reset {
    pub id: usize,
    pub room_name: String,
}

/// `ChessServer` manages chat rooms and responsible for coordinating chat session.
///
/// Implementation is very naïve.
#[derive(Debug)]
pub struct ChessServer {
    sessions: HashMap<usize, Recipient<Message>>,
    rooms: HashMap<String, Room>,
    rng: ThreadRng,
    visitor_count: Arc<AtomicUsize>,
}

#[derive(Debug)]
pub struct Room {
    original_fen: String,
    current_fen: String,
    moves: Vec<String>,
    sessions: HashSet<usize>,
    original_trash: String,
    trash: String,
    empty_at: Option<Instant>,
}

impl Room {
    pub fn new(fen: Option<String>, trash: Option<String>) -> Self {
        let fen =
            fen.unwrap_or("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string());
        let trash = trash.unwrap_or("".to_string());

        Self {
            original_fen: fen.clone(),
            current_fen: fen,
            moves: vec![],
            sessions: HashSet::new(),
            trash: trash.clone(),
            original_trash: trash,
            empty_at: Some(Instant::now()),
        }
    }

    pub fn sessions(&self) -> &HashSet<usize> {
        &self.sessions
    }

    pub fn insert_session(&mut self, id: usize) {
        self.sessions.insert(id);
    }

    pub fn remove_session(&mut self, id: &usize) -> bool {
        self.sessions.remove(id)
    }
}

impl ChessServer {
    pub fn new(visitor_count: Arc<AtomicUsize>) -> ChessServer {
        // default room
        let mut rooms = HashMap::new();
        rooms.insert("main".to_owned(), Room::new(None, None));

        ChessServer {
            sessions: HashMap::new(),
            rooms,
            rng: rand::thread_rng(),
            visitor_count,
        }
    }
}

impl ChessServer {
    /// Send message to all users in the room
    fn send_message(&self, room_name: &str, message: &str, skip_id: usize) {
        if let Some(sessions) = self.rooms.get(room_name).map(|m| m.sessions()) {
            for id in sessions {
                if *id != skip_id {
                    if let Some(addr) = self.sessions.get(id) {
                        addr.do_send(Message(message.to_owned()));
                    }
                }
            }
        }
    }

    fn send_message_to_session(&self, id: usize, message: &str) {
        if let Some(addr) = self.sessions.get(&id) {
            addr.do_send(Message(message.to_owned()));
        }
    }
}

/// Make actor from `ChessServer`
impl Actor for ChessServer {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("Chess server started");

        let room_timeout = Duration::from_secs(
            // 60 * 5 = 300 -> 5 minutes
            env::var("ROOM_TIMEOUT")
                .unwrap_or("300".to_string())
                .parse::<u64>()
                .unwrap_or(300),
        );

        ctx.run_interval(Duration::from_secs(5), move |act, _| {
            let mut rooms = vec![];

            for (name, room) in &mut act.rooms {
                if let Some(empty_at) = room.empty_at {
                    if empty_at.elapsed() > room_timeout {
                        log::debug!("Room {} is empty, removing", name);
                        rooms.push(name.clone());
                    }
                }
            }

            for name in rooms {
                act.rooms.remove(&name);
            }
        });
    }
}

/// Handler for Connect message.
///
/// Register new session and assign unique id to this session
impl Handler<Connect> for ChessServer {
    type Result = MessageResult<Connect>;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        log::debug!("Someone joined");

        // notify all users in same room
        self.send_message("main", "Someone joined", 0);

        // register session with random id
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr);

        // auto join session to main room
        let current_room = self
            .rooms
            .entry("main".to_owned())
            .or_insert_with(|| Room::new(None, None));

        current_room.insert_session(id);
        current_room.empty_at = None;
        let current_fen = current_room.current_fen.clone();
        let trash = current_room.trash.clone();

        let count = self.visitor_count.fetch_add(1, Ordering::SeqCst);
        self.send_message("main", &format!("Total visitors {count}"), 0);

        // sync fen
        self.send_message_to_session(id, &format!("/sync_board {}|{}", current_fen, trash));

        // send id, and current fen back
        MessageResult((id, current_fen))
    }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for ChessServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        log::info!("Someone disconnected");

        let mut rooms: Vec<String> = Vec::new();

        // remove address
        if self.sessions.remove(&msg.id).is_some() {
            // remove session from all rooms
            for (name, current_room) in &mut self.rooms {
                if current_room.remove_session(&msg.id) {
                    rooms.push(name.to_owned());

                    if current_room.sessions().is_empty() {
                        current_room.empty_at = Some(Instant::now());
                    }
                }
            }
        }

        // send message to other users
        for room in rooms {
            self.send_message(&room, "Someone disconnected", 0);
        }
    }
}

/// Handler for Message message.
impl Handler<ClientMessage> for ChessServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        self.send_message(&msg.room_name, msg.msg.as_str(), msg.id);
    }
}

/// Handler for `ListRooms` message.
// impl Handler<ListMatches> for ChessServer {
//     type Result = MessageResult<ListMatches>;
//
//     fn handle(&mut self, _: ListMatches, _: &mut Context<Self>) -> Self::Result {
//         let mut rooms = Vec::new();
//
//         for key in self.rooms.keys() {
//             rooms.push(key.to_owned())
//         }
//
//         MessageResult(rooms)
//     }
// }

/// Join room, send disconnect message to old room
/// send join message to new room
impl Handler<Join> for ChessServer {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Context<Self>) {
        let Join {
            id,
            name,
            fen,
            trash,
        } = msg;
        let mut rooms = Vec::new();

        // remove session from all rooms
        for (n, current_room) in &mut self.rooms {
            if current_room.remove_session(&id) {
                rooms.push(n.to_owned());

                if current_room.sessions().is_empty() {
                    current_room.empty_at = Some(Instant::now());
                }
            }
        }
        // send message to other users
        for room_name in rooms {
            self.send_message(&room_name, "Someone disconnected", 0);
        }

        let current_room = self
            .rooms
            .entry(name.clone())
            .or_insert_with(|| Room::new(fen, trash));

        current_room.insert_session(id);
        current_room.empty_at = None;
        let current_fen = current_room.current_fen.clone();
        let trash = current_room.trash.clone();

        self.send_message(&name, "Someone connected", id);

        // sync fen
        self.send_message_to_session(id, &format!("/sync_board {}|{}", current_fen, trash));
    }
}

impl Handler<Move> for ChessServer {
    type Result = ();

    fn handle(&mut self, msg: Move, _: &mut Self::Context) -> Self::Result {
        let Move {
            id,
            room_name,
            piece,
            from,
            to,
        } = msg;

        if let Some(current_room) = self.rooms.get_mut(&room_name) {
            let mut chessboard = ChessBoard::new(current_room.current_fen.as_str());
            chessboard.set_trash_from_str(current_room.trash.as_str());
            let from_position: Option<Position> = from.parse().ok();
            let to_position: Option<Position> = to.parse().ok();
            chessboard.move_piece(&piece, from_position, to_position);
            current_room.current_fen = chessboard.fen.clone();
            current_room.trash = chessboard.trash_string();

            let move_msg = format!("/move {} {} {}", piece, from, to);

            current_room.moves.push(move_msg.clone());

            self.send_message(&room_name, &move_msg, id);
        };
    }
}

impl Handler<Reset> for ChessServer {
    type Result = ();

    fn handle(&mut self, msg: Reset, _: &mut Self::Context) -> Self::Result {
        let Reset { id: _, room_name } = msg;

        if let Some(current_room) = self.rooms.get_mut(&room_name) {
            current_room.current_fen = current_room.original_fen.clone();
            current_room.trash = current_room.original_trash.to_owned();

            let current_fen = current_room.current_fen.clone();
            let trash = current_room.trash.clone();

            self.send_message(
                &room_name,
                &format!("/sync_board {}|{}", current_fen, trash),
                0,
            );
        };
    }
}
