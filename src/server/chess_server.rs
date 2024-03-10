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

use crate::entities::{
    chess_board::{self, enums::CastlePosition, turns::Turn, ChessBoard, ChessBoardBuilder},
    position::Position,
    stone::Stone,
};

use super::websockets::session::WsChessSession;

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
    pub addr: Addr<WsChessSession>,
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
}

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
    pub piece: String,
    pub from: String,
    pub to: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Reset {
    pub id: String,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct Undo {
    pub id: String,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct Redo {
    pub id: String,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct UserSync {
    pub id: String,
    pub name: String,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct Options {
    pub id: String,
    pub validation: bool,
    pub sync: bool,
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
    pub addr: Addr<WsChessSession>,
    pub current_room: String,
    pub disconected_at: Option<Instant>,
}

impl User {
    pub fn new(
        id: String,
        name: String,
        addr: Addr<WsChessSession>,
        current_room: String,
        disconected_at: Option<Instant>,
    ) -> Self {
        Self {
            id,
            name,
            addr,
            current_room,
            disconected_at,
        }
    }

    pub fn to_string(&self) -> String {
        let status = if self.disconected_at.is_some() {
            "away"
        } else {
            "online"
        };
        format!("{}:{}:{}", self.id, self.name, status)
    }
}

#[derive(Clone, Debug)]
pub struct MoveResult {
    pub from: Option<Position>,
    pub to: Option<Position>,
    pub stone: Stone,
    pub chess_board_move: chess_board::enums::Move,
    pub msg: String,
    pub previous_fen: String,
    pub previous_trash: String,
    pub current_fen: String,
    pub current_trash: String,
}

#[derive(Debug)]
pub struct Room {
    original_fen: String,
    current_fen: String,
    chess_board: ChessBoard,
    moves: Vec<MoveResult>,
    sessions: HashMap<String, User>,
    original_trash: String,
    trash: String,
    empty_at: Option<Instant>,
    current_move_index: Option<usize>,
}

impl Room {
    pub fn new(fen: Option<String>, trash: Option<String>) -> Result<Self, ()> {
        let fen =
            fen.unwrap_or("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string());
        let trash = trash.unwrap_or("".to_string());
        let chess_board = ChessBoardBuilder::new()
            .fen(&fen)
            .deleted_stones(&trash)
            .validation(false)
            .sync(true)
            .build()
            .map_err(|_| ())?;

        Ok(Self {
            original_fen: fen.clone(),
            current_fen: fen.clone(),
            chess_board,
            moves: vec![],
            sessions: HashMap::new(),
            trash: trash.clone(),
            original_trash: trash,
            empty_at: Some(Instant::now()),
            current_move_index: None,
        })
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

    pub fn disconnect_session(&mut self, id: &str) {
        if let Some(user) = self.sessions.get_mut(id) {
            user.disconected_at = Some(Instant::now());
        }
    }

    pub fn connect_session(&mut self, id: &str, addr: Addr<WsChessSession>) {
        if let Some(user) = self.sessions.get_mut(id) {
            user.disconected_at = None;
            user.addr = addr;
        }
    }

    pub fn usernames(&self) -> Vec<String> {
        self.sessions
            .values()
            .map(|user| user.to_string())
            .collect()
    }

    pub fn options_string(&self) -> String {
        let validation = self.chess_board.validation;
        let sync = self.chess_board.sync;
        let mut str = String::new();

        if validation {
            str.push_str(" validation");
        }

        if sync {
            str.push_str(" sync");
        }

        str.trim().to_string()
    }

    pub fn push_move(&mut self, result: MoveResult) {
        self.moves.push(result);
        match self.current_move_index {
            Some(_) => {
                self.current_move_index = Some(self.moves.len() - 1);
            }
            None => {
                self.current_move_index = Some(0);
            }
        }
    }

    pub fn get_move(&self, idx: &usize) -> Option<MoveResult> {
        self.moves.get(idx.clone()).cloned()
    }

    pub fn undo_move(&mut self) -> Result<MoveResult, ()> {
        if let Some(i) = self.current_move_index {
            let result = self.get_move(&i);

            if i > 0 {
                self.current_move_index = Some(i - 1);
            } else {
                self.current_move_index = None;
            }

            result.ok_or(())
        } else {
            Err(())
        }
    }

    pub fn redo_move(&mut self) -> Result<MoveResult, ()> {
        if self.current_move_index.map(|i| i as isize).unwrap_or(-1) < self.moves.len() as isize - 1
        {
            match self.current_move_index {
                Some(i) => {
                    self.current_move_index = Some(i + 1);
                }
                None => {
                    self.current_move_index = Some(0);
                }
            }

            self.get_move(&self.current_move_index.unwrap()).ok_or(())
        } else {
            Err(())
        }
    }

    pub fn truncate_moves_on_current_move(&mut self) {
        if let Some(i) = self.current_move_index {
            self.moves.truncate(i + 1);
        } else {
            self.moves.truncate(0);
        }
    }
}

impl ChessServer {
    pub fn new(visitor_count: Arc<AtomicUsize>) -> ChessServer {
        // default room
        let mut rooms = HashMap::new();
        rooms.insert(
            "main".to_owned(),
            Room::new(None, None).expect("Failed to create default room"),
        );

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
                    if let Some(user) = self.sessions.get(id) {
                        user.addr.do_send(Message(message.to_owned()));
                    }
                }
            }
        }
    }

    fn send_message_to_session(&self, id: &str, message: &str) {
        if let Some(user) = self.sessions.get(id) {
            user.addr.do_send(Message(message.to_owned()));
        }
    }

    fn disconnect_session(&mut self, id: &str) -> Option<&mut User> {
        if let Some(user) = self.sessions.get_mut(id) {
            user.disconected_at = Some(Instant::now());

            if let Some(room) = self.rooms.get_mut(&user.current_room) {
                room.disconnect_session(id);
            }

            Some(user)
        } else {
            None
        }
    }

    fn remove_session(&mut self, id: &str) -> Option<User> {
        let user = self.sessions.remove(id);

        if let Some(u) = user.as_ref() {
            if let Some(room) = self.rooms.get_mut(&u.current_room) {
                room.remove_session(id);

                if room.sessions().is_empty() {
                    room.empty_at = Some(Instant::now());
                }
            }
        }

        user
    }

    fn connect_session(&mut self, id: &str, addr: Addr<WsChessSession>) -> Option<&mut User> {
        if let Some(user) = self.sessions.get_mut(id) {
            user.addr.do_send(Disconnect { id: id.to_string() });

            user.disconected_at = None;
            user.addr = addr.clone();

            if let Some(room) = self.rooms.get_mut(&user.current_room) {
                room.connect_session(id, addr);
            }

            Some(user)
        } else {
            None
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

        let user_timeout = Duration::from_secs(
            // 60 * 5 = 300 -> 5 minutes
            env::var("USER_TIMEOUT")
                .unwrap_or("300".to_string())
                .parse::<u64>()
                .unwrap_or(300),
        );

        ctx.run_interval(Duration::from_secs(5), move |act, _| {
            let mut rooms = vec![];
            let mut sessions = vec![];

            for (name, room) in &mut act.rooms {
                if let Some(empty_at) = room.empty_at {
                    if empty_at.elapsed() > room_timeout {
                        log::info!("Room {} is empty, removing", name);
                        rooms.push(name.clone());
                    }
                }
            }

            for (id, user) in &mut act.sessions {
                if let Some(disconected_at) = user.disconected_at {
                    if disconected_at.elapsed() > user_timeout {
                        log::info!("User {}:{} is disconected, removing", id, user.name);
                        sessions.push(id.clone());
                    }
                }
            }

            for name in rooms {
                act.rooms.remove(&name);
            }

            for id in sessions {
                if let Some(user) = act.remove_session(&id) {
                    act.send_message(
                        &user.current_room,
                        &format!("/remove_user {}", user.to_string()),
                        None,
                    );
                };
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

        // register session with random id
        let Connect { addr, id, name } = msg;
        if let Some(user) = self.connect_session(&id, addr.clone()).cloned() {
            let user_string = user.to_string();
            let room_name = user.current_room.clone();

            if let Some(current_room) = self.rooms.get(&user.current_room) {
                let is_checkmate = current_room.chess_board.is_checkmate();
                let current_fen = current_room.current_fen.clone();
                let trash = current_room.trash.clone();
                let users = current_room.usernames().join(",");
                let options_str = current_room.options_string();

                // send message to all users in the room
                self.send_message(
                    &room_name,
                    &format!("/connect_user {}", user_string),
                    Some(&id),
                );

                // sync fen
                self.send_message_to_session(
                    &id,
                    &format!("/sync_board {}|{}|{}", room_name, current_fen, trash),
                );
                // sync users
                self.send_message_to_session(&id, &format!("/sync_users {}|{}", room_name, users));
                // sync options
                self.send_message_to_session(&id, &format!("/sync_options {}", options_str));
                // notify user if checkmate
                if is_checkmate {
                    self.send_message_to_session(&id, "/checkmate");
                }
            }
        } else {
            let room_name = "main".to_string();
            let user = User::new(id.clone(), name, addr, room_name.clone(), None);

            let user_string = user.to_string();
            self.sessions.insert(id.clone(), user.clone());
            // auto join session to main room
            let current_room = self
                .rooms
                .entry(room_name.clone())
                .or_insert_with(|| Room::new(None, None).expect("Failed to create default room"));

            current_room.insert_session(id.clone(), user);
            current_room.empty_at = None;
            let current_fen = current_room.current_fen.clone();
            let trash = current_room.trash.clone();
            let users = current_room.usernames().join(",");
            let options_str = current_room.options_string();

            let count = self.visitor_count.fetch_add(1, Ordering::SeqCst);
            self.send_message(&room_name, &format!("Total visitors {count}"), None);

            // send message to all users in the room
            self.send_message(&room_name, &format!("/add_user {}", user_string), Some(&id));

            // sync fen
            self.send_message_to_session(
                &id,
                &format!("/sync_board {}|{}|{}", room_name, current_fen, trash),
            );
            // sync users
            self.send_message_to_session(&id, &format!("/sync_users {}|{}", room_name, users));
            // sync options
            self.send_message_to_session(&id, &format!("/sync_options {}", options_str));
        }
    }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for ChessServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        log::info!("Someone disconnected");

        let mut rooms: Vec<(String, String)> = Vec::new();

        // remove address
        if let Some(user) = self.disconnect_session(&msg.id) {
            rooms.push((
                user.current_room.clone(),
                format!("/disconnect_user {}", user.to_string()),
            ));
        }

        // send message to all users in all rooms
        for (room_name, message) in rooms {
            self.send_message(&room_name, &message, Some(&msg.id));
        }
    }
}

/// Handler for Message message.
impl Handler<ClientMessage> for ChessServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        let Some(session) = self.sessions.get(&msg.id) else {
            log::error!("No user found for id {}", msg.id);
            return;
        };

        self.send_message(&session.current_room, msg.msg.as_str(), Some(&msg.id));
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
        let Some(user) = self.sessions.get_mut(&id) else {
            log::error!("No user found for id {}", id);
            return;
        };

        user.current_room = name.clone();
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

        let current_room = self.rooms.get_mut(&name);

        let current_room = if current_room.is_some() {
            current_room.unwrap()
        } else {
            let Ok(room) = Room::new(fen.clone(), trash.clone()) else {
                self.send_message_to_session(&id, "/notify error Failed to create room");
                return;
            };
            self.rooms.entry(name.clone()).or_insert_with(|| room)
        };

        current_room.insert_session(id.clone(), user.clone());
        current_room.empty_at = None;
        let current_fen = current_room.current_fen.clone();
        let trash = current_room.trash.clone();
        let users = current_room.usernames().join(",");
        let options_str = current_room.options_string();

        // send message to all users in all rooms
        for (room_name, message) in rooms {
            self.send_message(&room_name, &message, None);
        }
        // sync fen
        self.send_message_to_session(
            &id,
            &format!("/sync_board {}|{}|{}", name, current_fen, trash),
        );
        // sync users
        self.send_message_to_session(&id, &format!("/sync_users {}|{}", name, users));
        // sync options
        self.send_message_to_session(&id, &format!("/sync_options {}", options_str));
        // notify user
        self.send_message_to_session(&id, &format!("/notify success Joined room {}", name));

        // notify all users in room
        self.send_message(&name, &format!("/add_user {}", user_string), Some(&id));
    }
}

impl Handler<Move> for ChessServer {
    type Result = ();

    fn handle(&mut self, msg: Move, _: &mut Self::Context) -> Self::Result {
        let Move {
            id,
            piece,
            from,
            to,
        } = msg;

        let Some(session) = self.sessions.get(&id) else {
            log::error!("No user found for id {}", id);
            return;
        };

        if let Some(current_room) = self.rooms.get_mut(&session.current_room) {
            let chess_board = &mut current_room.chess_board;
            let from_position: Option<Position> = from.parse().ok();
            let to_position: Option<Position> = to.parse().ok();
            let current_fen = chess_board.fen.clone();
            let trash = chess_board.trash_string();
            let mut reactive_move_message = (0, None);
            let chess_board_move_result;
            match chess_board.move_piece(&piece, from_position.clone(), to_position.clone()) {
                Ok(move_result) => {
                    chess_board_move_result = Some(move_result.clone());
                    match move_result {
                        chess_board::enums::Move::Passant => {
                            let to = to_position.clone().unwrap();
                            let (piece, passant_pos) = match chess_board.turn {
                                Turn::White => ("lp", Position::new(to.x, to.y - 1)),
                                Turn::Black => ("dp", Position::new(to.x, to.y + 1)),
                            };
                            reactive_move_message = (
                                0,
                                Some(format!(
                                    "/move {} {} deleted",
                                    piece,
                                    passant_pos.to_string()
                                )),
                            );
                        }
                        chess_board::enums::Move::Castle(castle_side) => {
                            let (old_rook_x, new_rook_x) = match castle_side {
                                CastlePosition::KingSide => (7, 5),
                                CastlePosition::QueenSide => (0, 3),
                            };
                            let (piece, rook_pos) = match chess_board.turn {
                                Turn::White => ("dr", Position::new(old_rook_x, 0)),
                                Turn::Black => ("lr", Position::new(old_rook_x, 7)),
                            };
                            reactive_move_message = (
                                0,
                                Some(format!(
                                    "/move {} {} {}",
                                    piece,
                                    rook_pos.to_string(),
                                    Position::new(new_rook_x, rook_pos.y).to_string()
                                )),
                            );
                        }
                        chess_board::enums::Move::Promotion(_) => {
                            reactive_move_message = (
                                200,
                                Some(format!(
                                    "/sync_board {}|{}|{}",
                                    session.current_room,
                                    chess_board.fen.clone(),
                                    chess_board.trash_string()
                                )),
                            );
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    log::warn!(
                        "Room: {} -> failed attempt to move piece {} from {} to {} -> {:?}",
                        session.current_room,
                        piece,
                        from,
                        to,
                        e
                    );
                    self.send_message_to_session(
                        &id,
                        &format!(
                            "/sync_board {}|{}|{}",
                            session.current_room, current_fen, trash
                        ),
                    );
                    return;
                }
            }
            let is_checkmate = chess_board.is_checkmate();
            current_room.current_fen = chess_board.fen.clone();
            current_room.trash = chess_board.trash_string();

            let move_msg = format!("/move {} {} {}", piece, from, to);
            let move_result = MoveResult {
                stone: piece.parse().unwrap(),
                from: from_position,
                to: to_position,
                chess_board_move: chess_board_move_result.unwrap(),
                msg: move_msg.clone(),
                previous_fen: current_fen,
                previous_trash: trash,
                current_fen: current_room.current_fen.clone(),
                current_trash: current_room.trash.clone(),
            };
            current_room.truncate_moves_on_current_move();
            current_room.push_move(move_result);

            self.send_message(&session.current_room, &move_msg, Some(&id));
            if let (timeout, Some(reactive_move_message)) = reactive_move_message {
                std::thread::sleep(Duration::from_millis(timeout));
                self.send_message(&session.current_room, &reactive_move_message, None);
            }
            if is_checkmate {
                self.send_message(&session.current_room, "/checkmate", None)
            }
        };
    }
}

impl Handler<Reset> for ChessServer {
    type Result = ();

    fn handle(&mut self, msg: Reset, _: &mut Self::Context) -> Self::Result {
        let Some(session) = self.sessions.get(&msg.id) else {
            log::error!("No user found for id {}", msg.id);
            return;
        };

        if let Some(current_room) = self.rooms.get_mut(&session.current_room) {
            let Ok(chess_board) = ChessBoardBuilder::new()
                .fen(&current_room.original_fen)
                .deleted_stones(&current_room.original_trash)
                .validation(current_room.chess_board.validation)
                .sync(current_room.chess_board.sync)
                .build()
            else {
                self.send_message_to_session(&msg.id, "/notify error Failed to reset board");
                return;
            };
            current_room.current_fen = current_room.original_fen.clone();
            current_room.trash = current_room.original_trash.to_owned();
            current_room.current_move_index = None;
            current_room.chess_board = chess_board;

            let current_fen = current_room.current_fen.clone();
            let trash = current_room.trash.clone();

            self.send_message(
                &session.current_room,
                &format!(
                    "/sync_board {}|{}|{}",
                    session.current_room, current_fen, trash
                ),
                None,
            );
        };
    }
}

impl Handler<Undo> for ChessServer {
    type Result = ();

    fn handle(&mut self, msg: Undo, _: &mut Self::Context) -> Self::Result {
        let Undo { id } = msg.clone();

        let Some(session) = self.sessions.get(&id) else {
            log::error!("No user found for id {}", id);
            return;
        };

        if let Some(current_room) = self.rooms.get_mut(&session.current_room) {
            let msg;

            match current_room.undo_move() {
                Ok(move_result) => {
                    msg = format!(
                        "/sync_board {}|{}|{}",
                        session.current_room, move_result.previous_fen, move_result.previous_trash
                    );
                    let Ok(chess_board) = ChessBoardBuilder::new()
                        .fen(&move_result.previous_fen)
                        .deleted_stones(&move_result.previous_trash)
                        .validation(current_room.chess_board.validation)
                        .sync(current_room.chess_board.sync)
                        .build()
                    else {
                        let _ = current_room.redo_move();
                        self.send_message_to_session(&id, "/notify error Failed to undo move");
                        return;
                    };
                    current_room.current_fen = move_result.previous_fen;
                    current_room.trash = move_result.previous_trash;
                    current_room.chess_board = chess_board;
                }
                Err(_) => {
                    let fen = current_room.current_fen.clone();
                    let trash = current_room.trash.clone();
                    self.send_message_to_session(&id, "/notify warning No more moves to undo");
                    msg = format!("/sync_board {}|{}|{}", session.current_room, fen, trash);
                }
            };

            self.send_message(&session.current_room, &msg, None);
        } else {
            log::error!("No room found with name {}", session.current_room);
        }
    }
}

impl Handler<Redo> for ChessServer {
    type Result = ();

    fn handle(&mut self, msg: Redo, _: &mut Self::Context) -> Self::Result {
        let Redo { id } = msg.clone();

        let Some(session) = self.sessions.get(&id) else {
            log::error!("No user found for id {}", id);
            return;
        };

        if let Some(current_room) = self.rooms.get_mut(&session.current_room) {
            let msg;
            let mut is_checkmate = false;

            match current_room.redo_move() {
                Ok(move_result) => {
                    msg = format!(
                        "/sync_board {}|{}|{}",
                        session.current_room, move_result.current_fen, move_result.current_trash
                    );
                    let Ok(chess_board) = ChessBoardBuilder::new()
                        .fen(&move_result.current_fen)
                        .deleted_stones(&move_result.current_trash)
                        .validation(current_room.chess_board.validation)
                        .sync(current_room.chess_board.sync)
                        .build()
                    else {
                        let _ = current_room.undo_move();
                        self.send_message_to_session(&id, "/notify error Failed to redo move");
                        return;
                    };

                    is_checkmate = chess_board.is_checkmate();
                    current_room.current_fen = move_result.current_fen;
                    current_room.trash = move_result.current_trash;
                    current_room.chess_board = chess_board;
                }
                Err(_) => {
                    let fen = current_room.current_fen.clone();
                    let trash = current_room.trash.clone();
                    self.send_message_to_session(&id, "/notify warning No more moves to redo");
                    msg = format!("/sync_board {}|{}|{}", session.current_room, fen, trash);
                }
            };

            self.send_message(&session.current_room, &msg, None);
            if is_checkmate {
                self.send_message(&session.current_room, "/checkmate", None)
            }
        } else {
            log::error!("No room found with name {}", session.current_room);
        }
    }
}

impl Handler<UserSync> for ChessServer {
    type Result = ();

    fn handle(&mut self, msg: UserSync, _: &mut Self::Context) -> Self::Result {
        let UserSync { id, name } = msg.clone();

        let Some(user) = self.sessions.get_mut(&id) else {
            log::error!("No user found for id {}", id);
            return;
        };

        user.name = name.clone();
        let addr = user.addr.clone();

        let user_string = user.to_string();
        let current_room_name = user.current_room.clone();

        if let Some(current_room) = self.rooms.get_mut(&current_room_name) {
            current_room
                .sessions
                .get_mut(&id)
                .expect(&format!(
                    "User {}:{} was expected to be in the room {}",
                    user.name, user.id, current_room_name
                ))
                .name = name;

            // notify all users in room
            self.send_message(
                &current_room_name,
                &format!("/add_user {}", user_string),
                None,
            );
        };
        addr.do_send(msg);
    }
}

impl Handler<Options> for ChessServer {
    type Result = ();

    fn handle(&mut self, msg: Options, _: &mut Self::Context) -> Self::Result {
        let Some(session) = self.sessions.get(&msg.id) else {
            log::error!("No user found for id {}", msg.id);
            return;
        };

        let Some(current_room) = self.rooms.get_mut(&session.current_room) else {
            log::error!("No room found with name {}", session.current_room);
            return;
        };

        let new_chess_board = ChessBoardBuilder::new()
            .fen(&current_room.current_fen)
            .deleted_stones(&current_room.trash)
            .validation(msg.validation)
            .sync(msg.sync)
            .build();

        let result_msg;

        if let Ok(chess_board) = new_chess_board {
            current_room.chess_board = chess_board;

            result_msg = current_room.options_string();
            self.send_message_to_session(&msg.id, "/notify success Options applied");
        } else {
            result_msg = current_room.options_string();
            self.send_message_to_session(&msg.id, "/notify error Failed to apply options");
        }

        self.send_message(
            &session.current_room,
            &format!("/sync_options {}", result_msg),
            None,
        );
    }
}
