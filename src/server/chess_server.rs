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

use crate::entities::{
    chess_board::{
        enums::{CastlePosition, ChessBoardError, Move as BoardMove, MoveError},
        ChessBoard, ChessBoardBuilder,
    },
    position::{Position, DELETED},
    stone::{Color, Stone},
};

use super::websockets::session::WsChessSession;

const MAIN_ROOM: &str = "main";
const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
/// Gives clients time to animate a promoting pawn before its board is replaced
const PROMOTION_SYNC_DELAY: Duration = Duration::from_millis(200);

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
    /// Socket that closed, ignored when the user already reconnected on another one
    pub addr: Addr<WsChessSession>,
}

/// Tells a session it was replaced by a newer socket of the same user
#[derive(Message)]
#[rtype(result = "()")]
pub struct Kick;

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

#[derive(Clone, Debug)]
pub struct User {
    pub id: String,
    pub name: String,
    pub addr: Addr<WsChessSession>,
    pub current_room: String,
    pub disconnected_at: Option<Instant>,
}

impl User {
    pub fn new(id: String, name: String, addr: Addr<WsChessSession>, current_room: String) -> Self {
        Self {
            id,
            name,
            addr,
            current_room,
            disconnected_at: None,
        }
    }

    pub fn to_string(&self) -> String {
        let status = if self.disconnected_at.is_some() {
            "away"
        } else {
            "online"
        };
        format!("{}:{}:{}", self.id, self.name, status)
    }
}

/// A move played on a room's board, and what to broadcast for it
struct PlayedMove {
    message: String,
    /// Side effects the clients don't apply themselves (the pawn taken en
    /// passant, the castling rook) and checkmate
    follow_ups: Vec<String>,
    /// A promotion swaps the pawn, so clients get the whole board once they're
    /// done animating the move
    resync: bool,
}

#[derive(Debug)]
pub struct Room {
    name: String,
    chess_board: ChessBoard,
    /// Every `(fen, trash)` the board went through, starting with the initial one
    history: Vec<(String, String)>,
    /// Index in `history` of the position on the board, moved by undo and redo
    history_index: usize,
    /// Ids of the users in the room
    sessions: HashSet<String>,
    empty_at: Option<Instant>,
}

fn build_board(
    fen: &str,
    trash: &str,
    validation: bool,
    sync: bool,
) -> Result<ChessBoard, ChessBoardError> {
    ChessBoardBuilder::new()
        .fen(fen)
        .deleted_stones(trash)
        .validation(validation)
        .sync(sync)
        .build()
}

impl Room {
    pub fn new(
        name: &str,
        fen: Option<String>,
        trash: Option<String>,
    ) -> Result<Self, ChessBoardError> {
        let fen = fen.unwrap_or(DEFAULT_FEN.to_string());
        let trash = trash.unwrap_or_default();
        let chess_board = build_board(&fen, &trash, false, true)?;

        Ok(Self {
            name: name.to_string(),
            chess_board,
            history: vec![(fen, trash)],
            history_index: 0,
            sessions: HashSet::new(),
            empty_at: Some(Instant::now()),
        })
    }

    pub fn sync_board_message(&self) -> String {
        format!(
            "/sync_board {}|{}|{}",
            self.name,
            self.chess_board.fen,
            self.chess_board.trash_string()
        )
    }

    pub fn options_string(&self) -> String {
        let mut str = String::new();

        if self.chess_board.validation {
            str.push_str(" validation");
        }

        if self.chess_board.sync {
            str.push_str(" sync");
        }

        str.trim().to_string()
    }

    pub fn set_options(&mut self, validation: bool, sync: bool) -> Result<(), ChessBoardError> {
        self.chess_board = build_board(
            &self.chess_board.fen,
            &self.chess_board.trash_string(),
            validation,
            sync,
        )?;
        Ok(())
    }

    /// Loads `history[index]`, keeping the room's options
    fn load_history(&mut self, index: usize) -> Result<(), ChessBoardError> {
        let (fen, trash) = self.history.get(index).ok_or(ChessBoardError::BuildError)?;
        self.chess_board = build_board(
            fen,
            trash,
            self.chess_board.validation,
            self.chess_board.sync,
        )?;
        self.history_index = index;
        Ok(())
    }

    /// Back to the initial position. Redo still replays the undone moves.
    pub fn reset(&mut self) -> Result<(), ChessBoardError> {
        self.load_history(0)
    }

    /// `Ok(false)` when there is no move to undo
    pub fn undo(&mut self) -> Result<bool, ChessBoardError> {
        if self.history_index == 0 {
            return Ok(false);
        }
        self.load_history(self.history_index - 1).map(|_| true)
    }

    /// `Ok(false)` when there is no move to redo
    pub fn redo(&mut self) -> Result<bool, ChessBoardError> {
        if self.history_index + 1 >= self.history.len() {
            return Ok(false);
        }
        self.load_history(self.history_index + 1).map(|_| true)
    }

    /// Plays a `/move` from a client. `Ok(None)` when the stone didn't move.
    fn play_move(
        &mut self,
        piece: &str,
        from: &str,
        to: &str,
    ) -> Result<Option<PlayedMove>, ChessBoardError> {
        let invalid_move = || ChessBoardError::InvalidMove(MoveError::InvalidMove);
        let stone = piece
            .parse::<Stone>()
            .map_err(|_| ChessBoardError::InvalidMove(MoveError::NoStoneFound))?;
        let from_position = Position::parse_square(from).map_err(|_| invalid_move())?;
        let to_position = Position::parse_square(to).map_err(|_| invalid_move())?;

        if from_position == to_position {
            return Ok(None);
        }

        let board_move = self
            .chess_board
            .move_piece(piece, from_position, to_position)?;

        let (rook, opponent_pawn, back_rank) = match stone.color() {
            Color::Light => ("lr", "dp", 7),
            Color::Dark => ("dr", "lp", 0),
        };
        let mut follow_ups = Vec::new();
        let mut resync = false;

        match board_move {
            BoardMove::Passant(captured) => {
                follow_ups.push(format!(
                    "/move {} {} {}",
                    opponent_pawn,
                    captured.to_string(),
                    DELETED
                ));
            }
            BoardMove::Castle(side) => {
                let (rook_from, rook_to) = match side {
                    CastlePosition::KingSide => (7, 5),
                    CastlePosition::QueenSide => (0, 3),
                };
                follow_ups.push(format!(
                    "/move {} {} {}",
                    rook,
                    Position::new(rook_from, back_rank).to_string(),
                    Position::new(rook_to, back_rank).to_string()
                ));
            }
            BoardMove::Promotion(_) => resync = true,
            BoardMove::Normal => {}
        }

        // The resync carries the checkmate itself, so it isn't wiped by the board reload
        if !resync && self.chess_board.is_checkmate() {
            follow_ups.push("/checkmate".to_string());
        }

        self.history.truncate(self.history_index + 1);
        self.history.push((
            self.chess_board.fen.clone(),
            self.chess_board.trash_string(),
        ));
        self.history_index += 1;

        Ok(Some(PlayedMove {
            message: format!("/move {} {} {}", piece, from, to),
            follow_ups,
            resync,
        }))
    }
}

impl ChessServer {
    pub fn new(visitor_count: Arc<AtomicUsize>) -> ChessServer {
        // default room
        let mut rooms = HashMap::new();
        rooms.insert(
            MAIN_ROOM.to_owned(),
            Room::new(MAIN_ROOM, None, None).expect("Failed to create default room"),
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
        let Some(room) = self.rooms.get(room_name) else {
            return;
        };

        for id in &room.sessions {
            if Some(id.as_str()) != skip_id {
                self.send_message_to_session(id, message);
            }
        }
    }

    fn send_message_to_session(&self, id: &str, message: &str) {
        if let Some(user) = self.sessions.get(id) {
            user.addr.do_send(Message(message.to_owned()));
        }
    }

    /// Name of the room the user is in
    fn room_of(&self, id: &str) -> Option<String> {
        let room_name = self.sessions.get(id).map(|user| user.current_room.clone());
        if room_name.is_none() {
            log::error!("No user found for id {}", id);
        }
        room_name
    }

    fn users_string(&self, room: &Room) -> String {
        room.sessions
            .iter()
            .filter_map(|id| self.sessions.get(id))
            .map(User::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    /// Sends a session everything it needs to render its room
    fn sync_session(&self, id: &str, room_name: &str) {
        let Some(room) = self.rooms.get(room_name) else {
            log::error!("No room found with name {}", room_name);
            return;
        };

        self.send_message_to_session(id, &room.sync_board_message());
        self.send_message_to_session(
            id,
            &format!("/sync_users {}|{}", room.name, self.users_string(room)),
        );
        self.send_message_to_session(id, &format!("/sync_options {}", room.options_string()));
        if room.chess_board.is_checkmate() {
            self.send_message_to_session(id, "/checkmate");
        }
    }

    /// Sends the room's board to all its users
    fn broadcast_board(&self, room_name: &str) {
        let Some(room) = self.rooms.get(room_name) else {
            return;
        };

        self.send_message(room_name, &room.sync_board_message(), None);
        if room.chess_board.is_checkmate() {
            self.send_message(room_name, "/checkmate", None);
        }
    }

    fn enter_room(&mut self, id: &str, room_name: &str) {
        if let Some(room) = self.rooms.get_mut(room_name) {
            room.sessions.insert(id.to_string());
            room.empty_at = None;
        }

        if let Some(user) = self.sessions.get_mut(id) {
            user.current_room = room_name.to_string();
        }
    }

    fn leave_room(&mut self, id: &str) {
        let Some(user) = self.sessions.get(id) else {
            return;
        };
        let room_name = user.current_room.clone();
        let user_string = user.to_string();

        if let Some(room) = self.rooms.get_mut(&room_name) {
            room.sessions.remove(id);

            if room.sessions.is_empty() {
                room.empty_at = Some(Instant::now());
            }
        }

        self.send_message(&room_name, &format!("/remove_user {}", user_string), None);
    }

    fn remove_session(&mut self, id: &str) {
        self.leave_room(id);
        self.sessions.remove(id);
    }

    fn step_history(
        &mut self,
        id: &str,
        step: fn(&mut Room) -> Result<bool, ChessBoardError>,
        action: &str,
    ) {
        let Some(room_name) = self.room_of(id) else {
            return;
        };
        let Some(room) = self.rooms.get_mut(&room_name) else {
            return;
        };

        match step(room) {
            Ok(true) => self.broadcast_board(&room_name),
            Ok(false) => self.send_message_to_session(
                id,
                &format!("/notify warning No more moves to {}", action),
            ),
            Err(_) => {
                self.send_message_to_session(id, &format!("/notify error Failed to {} move", action))
            }
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
            act.rooms.retain(|name, room| {
                let expired = room
                    .empty_at
                    .is_some_and(|empty_at| empty_at.elapsed() > room_timeout);
                if expired {
                    log::info!("Room {} is empty, removing", name);
                }
                !expired
            });

            let expired_sessions: Vec<String> = act
                .sessions
                .values()
                .filter(|user| {
                    user.disconnected_at
                        .is_some_and(|disconnected_at| disconnected_at.elapsed() > user_timeout)
                })
                .map(|user| {
                    log::info!("User {}:{} is disconected, removing", user.id, user.name);
                    user.id.clone()
                })
                .collect();

            for id in expired_sessions {
                act.remove_session(&id);
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

        let Connect { addr, id, name } = msg;

        if let Some(user) = self.sessions.get_mut(&id) {
            // Same user on a new socket (page reload or another tab)
            user.addr.do_send(Kick);
            user.addr = addr;
            user.disconnected_at = None;

            let room_name = user.current_room.clone();
            let user_string = user.to_string();

            self.send_message(
                &room_name,
                &format!("/connect_user {}", user_string),
                Some(&id),
            );
            self.sync_session(&id, &room_name);
        } else {
            let user = User::new(id.clone(), name, addr, MAIN_ROOM.to_string());
            let user_string = user.to_string();
            self.sessions.insert(id.clone(), user);

            // auto join session to main room
            self.rooms.entry(MAIN_ROOM.to_string()).or_insert_with(|| {
                Room::new(MAIN_ROOM, None, None).expect("Failed to create default room")
            });
            self.enter_room(&id, MAIN_ROOM);

            let count = self.visitor_count.fetch_add(1, Ordering::SeqCst);
            self.send_message(MAIN_ROOM, &format!("Total visitors {count}"), None);
            self.send_message(MAIN_ROOM, &format!("/add_user {}", user_string), Some(&id));
            self.sync_session(&id, MAIN_ROOM);
        }
    }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for ChessServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        log::info!("Someone disconnected");

        let Some(user) = self.sessions.get_mut(&msg.id) else {
            return;
        };

        if user.addr != msg.addr {
            return;
        }

        user.disconnected_at = Some(Instant::now());
        let room_name = user.current_room.clone();
        let message = format!("/disconnect_user {}", user.to_string());

        self.send_message(&room_name, &message, Some(&msg.id));
    }
}

/// Handler for Message message.
impl Handler<ClientMessage> for ChessServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        let Some(room_name) = self.room_of(&msg.id) else {
            return;
        };

        self.send_message(&room_name, msg.msg.as_str(), Some(&msg.id));
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

        if !self.sessions.contains_key(&id) {
            log::error!("No user found for id {}", id);
            return;
        }

        // Create the room before leaving the current one, so a bad FEN leaves
        // the user where they were
        if !self.rooms.contains_key(&name) {
            let Ok(room) = Room::new(&name, fen, trash) else {
                self.send_message_to_session(&id, "/notify error Failed to create room");
                return;
            };
            self.rooms.insert(name.clone(), room);
        }

        self.leave_room(&id);
        self.enter_room(&id, &name);
        self.sync_session(&id, &name);
        self.send_message_to_session(&id, &format!("/notify success Joined room {}", name));

        // notify all users in room
        if let Some(user) = self.sessions.get(&id) {
            self.send_message(&name, &format!("/add_user {}", user.to_string()), Some(&id));
        }
    }
}

impl Handler<Move> for ChessServer {
    type Result = ();

    fn handle(&mut self, msg: Move, ctx: &mut Self::Context) -> Self::Result {
        let Move {
            id,
            piece,
            from,
            to,
        } = msg;

        let Some(room_name) = self.room_of(&id) else {
            return;
        };
        let Some(room) = self.rooms.get_mut(&room_name) else {
            return;
        };

        match room.play_move(&piece, &from, &to) {
            Ok(Some(played)) => {
                self.send_message(&room_name, &played.message, Some(&id));
                for follow_up in &played.follow_ups {
                    self.send_message(&room_name, follow_up, None);
                }
                if played.resync {
                    ctx.run_later(PROMOTION_SYNC_DELAY, move |act, _| {
                        act.broadcast_board(&room_name)
                    });
                }
            }
            Ok(None) => {}
            Err(e) => {
                log::warn!(
                    "Room: {} -> failed attempt to move piece {} from {} to {} -> {:?}",
                    room_name,
                    piece,
                    from,
                    to,
                    e
                );
                let sync_board = room.sync_board_message();
                self.send_message_to_session(&id, &sync_board);
            }
        }
    }
}

impl Handler<Reset> for ChessServer {
    type Result = ();

    fn handle(&mut self, msg: Reset, _: &mut Self::Context) -> Self::Result {
        let Some(room_name) = self.room_of(&msg.id) else {
            return;
        };
        let Some(room) = self.rooms.get_mut(&room_name) else {
            return;
        };

        match room.reset() {
            Ok(()) => self.broadcast_board(&room_name),
            Err(_) => self.send_message_to_session(&msg.id, "/notify error Failed to reset board"),
        }
    }
}

impl Handler<Undo> for ChessServer {
    type Result = ();

    fn handle(&mut self, msg: Undo, _: &mut Self::Context) -> Self::Result {
        self.step_history(&msg.id, Room::undo, "undo");
    }
}

impl Handler<Redo> for ChessServer {
    type Result = ();

    fn handle(&mut self, msg: Redo, _: &mut Self::Context) -> Self::Result {
        self.step_history(&msg.id, Room::redo, "redo");
    }
}

impl Handler<UserSync> for ChessServer {
    type Result = ();

    fn handle(&mut self, msg: UserSync, _: &mut Self::Context) -> Self::Result {
        let Some(user) = self.sessions.get_mut(&msg.id) else {
            log::error!("No user found for id {}", msg.id);
            return;
        };

        user.name = msg.name.clone();
        // keep the socket's copy of the name, used to prefix chat lines
        user.addr.do_send(msg);

        let room_name = user.current_room.clone();
        let message = format!("/add_user {}", user.to_string());

        // notify all users in room
        self.send_message(&room_name, &message, None);
    }
}

impl Handler<Options> for ChessServer {
    type Result = ();

    fn handle(&mut self, msg: Options, _: &mut Self::Context) -> Self::Result {
        let Some(room_name) = self.room_of(&msg.id) else {
            return;
        };
        let Some(room) = self.rooms.get_mut(&room_name) else {
            return;
        };

        let notification = match room.set_options(msg.validation, msg.sync) {
            Ok(()) => "/notify success Options applied",
            Err(_) => "/notify error Failed to apply options",
        };
        let options = format!("/sync_options {}", room.options_string());

        self.send_message_to_session(&msg.id, notification);
        self.send_message(&room_name, &options, None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn play(room: &mut Room, piece: &str, from: &str, to: &str) -> Option<PlayedMove> {
        room.play_move(piece, from, to).unwrap()
    }

    #[test]
    fn test_play_move_rejects_bad_input() {
        let mut room = Room::new("test", None, None).unwrap();
        let fen = room.chess_board.fen.clone();

        assert!(room.play_move("xx", "e2", "e4").is_err());
        assert!(room.play_move("lp", "i1", "e4").is_err());
        assert!(room.play_move("lp", "e2", "z9").is_err());
        assert!(room.play_move("lq", "e2", "e4").is_err());
        assert!(play(&mut room, "lp", "e2", "e2").is_none());
        assert_eq!(room.chess_board.fen, fen);
        assert_eq!(room.history.len(), 1);
    }

    #[test]
    fn test_history() {
        let mut room = Room::new("test", None, None).unwrap();
        let start = room.chess_board.fen.clone();

        play(&mut room, "lp", "e2", "e4").unwrap();
        let after_e4 = room.chess_board.fen.clone();
        play(&mut room, "dp", "e7", "e5").unwrap();

        assert_eq!(room.undo().unwrap(), true);
        assert_eq!(room.chess_board.fen, after_e4);
        assert_eq!(room.redo().unwrap(), true);
        assert_eq!(room.redo().unwrap(), false);

        room.reset().unwrap();
        assert_eq!(room.chess_board.fen, start);
        assert_eq!(room.undo().unwrap(), false);

        // A new move drops the undone ones
        play(&mut room, "lp", "d2", "d4").unwrap();
        assert_eq!(room.redo().unwrap(), false);
        assert_eq!(room.history.len(), 2);
    }

    fn validated_room(fen: &str) -> Room {
        let mut room = Room::new("test", Some(fen.to_string()), None).unwrap();
        room.set_options(true, true).unwrap();
        room
    }

    #[test]
    fn test_follow_ups() {
        let mut room = validated_room("r3k2r/8/8/3pP3/8/8/8/R3K2R w KQkq d6 0 1");

        let played = play(&mut room, "lp", "e5", "d6").unwrap();
        assert_eq!(played.follow_ups, vec!["/move dp d5 deleted"]);

        let played = play(&mut room, "dk", "e8", "c8").unwrap();
        assert_eq!(played.follow_ups, vec!["/move dr a8 d8"]);

        let mut room = validated_room("8/4P3/8/8/8/8/8/k6K w - - 0 1");
        let played = play(&mut room, "lp", "e7", "e8").unwrap();
        assert!(played.resync);
        assert!(played.follow_ups.is_empty());

        let mut room = validated_room("7k/8/6K1/8/8/8/8/5Q2 w - - 0 1");
        let played = play(&mut room, "lq", "f1", "f8").unwrap();
        assert_eq!(played.follow_ups, vec!["/checkmate"]);
    }
}
