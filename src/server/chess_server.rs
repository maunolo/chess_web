//! `ChessServer` is an actor. It maintains list of connection client session.
//! And manages available rooms. Peers send messages to other peers in same
//! room through `ChessServer`.

use std::{
    collections::HashMap,
    env,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use actix::prelude::*;

use crate::entities::{chess_board::ChessBoard, position::Position};

/// Chat server sends this messages to session
#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

/// Message for chat server communications

/// New chat session is created
#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub id: String,
    pub name: String,
    pub addr: Recipient<Message>,
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: String,
}

/// Send message to specific room
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    /// Id of the client session
    pub id: String,
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
    pub id: String,

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
    pub id: String,
    pub room_name: String,
    pub piece: String,
    pub from: String,
    pub to: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Reset {
    pub id: String,
    pub room_name: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UserSync {
    pub id: String,
    pub name: String,
}

/// `ChessServer` manages chat rooms and responsible for coordinating chat session.
///
/// Implementation is very naïve.
#[derive(Debug)]
pub struct ChessServer {
    sessions: HashMap<String, User>,
    rooms: HashMap<String, Room>,
    visitor_count: Arc<AtomicUsize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct User {
    pub id: String,
    pub name: String,
    pub recipient: Recipient<Message>,
    pub current_room: Option<String>,
}

impl User {
    pub fn new(
        id: String,
        name: String,
        recipient: Recipient<Message>,
        current_room: Option<String>,
    ) -> Self {
        Self {
            id,
            name,
            recipient,
            current_room,
        }
    }

    pub fn to_string(&self) -> String {
        format!("{}:{}", self.id, self.name)
    }
}

#[derive(Debug)]
pub struct Room {
    original_fen: String,
    current_fen: String,
    moves: Vec<String>,
    sessions: HashMap<String, User>,
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
            sessions: HashMap::new(),
            trash: trash.clone(),
            original_trash: trash,
            empty_at: Some(Instant::now()),
        }
    }

    pub fn sessions(&self) -> &HashMap<String, User> {
        &self.sessions
    }

    pub fn insert_session(&mut self, id: String, user: User) {
        self.sessions.insert(id, user);
    }

    pub fn remove_session(&mut self, id: &str) -> Option<User> {
        self.sessions.remove(id)
    }

    pub fn usernames(&self) -> Vec<String> {
        self.sessions
            .values()
            .map(|user| user.to_string())
            .collect()
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
            visitor_count,
        }
    }
}

impl ChessServer {
    /// Send message to all users in the room
    fn send_message(&self, room_name: &str, message: &str, skip_id: Option<&str>) {
        let skip_id = skip_id.unwrap_or("");
        if let Some(sessions) = self.rooms.get(room_name).map(|m| m.sessions()) {
            for (id, _) in sessions {
                if *id != skip_id {
                    if let Some(User {
                        id: _,
                        name: _,
                        recipient: addr,
                        current_room: _,
                    }) = self.sessions.get(id)
                    {
                        addr.do_send(Message(message.to_owned()));
                    }
                }
            }
        }
    }

    fn send_message_to_session(&self, id: &str, message: &str) {
        if let Some(User {
            id: _,
            name: _,
            recipient: addr,
            current_room: _,
        }) = self.sessions.get(id)
        {
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
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) {
        log::debug!("Someone joined");

        let room_name = "main".to_owned();

        // register session with random id
        let Connect { addr, id, name } = msg;
        let user = User::new(id.clone(), name, addr, Some(room_name.clone()));
        let user_string = user.to_string();
        self.sessions.insert(id.clone(), user.clone());

        // auto join session to main room
        let current_room = self
            .rooms
            .entry(room_name.clone())
            .or_insert_with(|| Room::new(None, None));

        current_room.insert_session(id.clone(), user);
        current_room.empty_at = None;
        let current_fen = current_room.current_fen.clone();
        let trash = current_room.trash.clone();
        let users = current_room.usernames().join(",");

        let count = self.visitor_count.fetch_add(1, Ordering::SeqCst);
        self.send_message(&room_name, &format!("Total visitors {count}"), None);

        // sync fen
        self.send_message_to_session(
            &id,
            &format!("/sync_board {}|{}|{}", room_name, current_fen, trash),
        );
        // sync users
        self.send_message_to_session(&id, &format!("/sync_users {}|{}", room_name, users));

        // send message to all users in the room
        self.send_message(&room_name, &format!("/add_user {}", user_string), Some(&id));
    }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for ChessServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        log::info!("Someone disconnected");

        let mut rooms: Vec<(String, String)> = Vec::new();

        // remove address
        if self.sessions.remove(&msg.id).is_some() {
            // remove session from all rooms
            for (name, current_room) in &mut self.rooms {
                if let Some(user) = current_room.remove_session(&msg.id) {
                    rooms.push((name.clone(), format!("/remove_user {}", user.to_string())));

                    if current_room.sessions().is_empty() {
                        current_room.empty_at = Some(Instant::now());
                    }
                }
            }

            // send message to all users in all rooms
            for (room_name, message) in rooms {
                self.send_message(&room_name, &message, None);
            }
        }
    }
}

/// Handler for Message message.
impl Handler<ClientMessage> for ChessServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        self.send_message(&msg.room_name, msg.msg.as_str(), Some(&msg.id));
    }
}

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
        let Some(user) = self.sessions.get(&id) else {
            log::error!("No user found for id {}", id);
            return;
        };

        let mut rooms: Vec<(String, String)> = Vec::new();
        let user_string = user.to_string();

        // remove session from all rooms
        for (n, current_room) in &mut self.rooms {
            if current_room.remove_session(&id).is_some() {
                rooms.push((n.clone(), format!("/remove_user {}", user_string)));

                if current_room.sessions().is_empty() {
                    current_room.empty_at = Some(Instant::now());
                }
            }
        }

        // send message to all users in all rooms
        for (room_name, message) in rooms {
            self.send_message(&room_name, &message, None);
        }

        let current_room = self
            .rooms
            .entry(name.clone())
            .or_insert_with(|| Room::new(fen, trash));

        current_room.insert_session(id.clone(), user.clone());
        current_room.empty_at = None;
        let current_fen = current_room.current_fen.clone();
        let trash = current_room.trash.clone();
        let users = current_room.usernames().join(",");

        // sync fen
        self.send_message_to_session(
            &id,
            &format!("/sync_board {}|{}|{}", name, current_fen, trash),
        );
        // sync users
        self.send_message_to_session(&id, &format!("/sync_users {}|{}", name, users));

        // notify all users in room
        self.send_message(&name, &format!("/add_user {}", user_string), Some(&id));
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

            self.send_message(&room_name, &move_msg, Some(&id));
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
                &format!("/sync_board {}|{}|{}", room_name, current_fen, trash),
                None,
            );
        };
    }
}

impl Handler<UserSync> for ChessServer {
    type Result = ();

    fn handle(&mut self, msg: UserSync, _: &mut Self::Context) -> Self::Result {
        let UserSync { id, name } = msg;

        let Some(user) = self.sessions.get_mut(&id) else {
            log::error!("No user found for id {}", id);
            return;
        };

        user.name = name.clone();
        if let Some(current_room_name) = user.current_room.clone() {
            let user_string = user.to_string();

            if let Some(current_room) = self.rooms.get_mut(&current_room_name) {
                current_room.sessions.get_mut(&id).unwrap().name = name;

                // notify all users in room
                self.send_message(
                    &current_room_name,
                    &format!("/remove_user {}", user_string),
                    Some(&id),
                );
                self.send_message(
                    &current_room_name,
                    &format!("/add_user {}", user_string),
                    Some(&id),
                );
            };
        };
    }
}
