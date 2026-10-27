//! `ChessServer` is an actor. It maintains list of connection client session.
//! And manages available rooms. Peers send messages to other peers in same
//! room through `ChessServer`.

use std::{
    collections::{HashMap, HashSet},
    env,
    time::{Duration, Instant},
};

use actix::prelude::*;

use crate::entities::{
    chess_board::{
        enums::{CastlePosition, ChessBoardError, Move as BoardMove, MoveError, PromotionKind},
        ChessBoard, ChessBoardBuilder,
    },
    notification::NotifyType,
    position::{Position, DELETED},
    protocol::{
        ClientMsg, ClockInfo, GameInfo, GameResult, Presence, SeatChoice, ServerMsg, Side,
        TimeControl, UserInfo, MAIN_ROOM,
    },
    stone::{Color, Stone},
};
use crate::utils::sanitize::{sanitize_room_name, sanitize_username};

use super::websockets::session::{Connection, Outbound};

const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
/// Gives clients time to animate a promoting pawn before its board is replaced
const PROMOTION_SYNC_DELAY: Duration = Duration::from_millis(200);
/// Longest clock a game can start with: 3 hours, plus at most 3 minutes per move
const MAX_INITIAL_SECS: u32 = 3 * 60 * 60;
const MAX_INCREMENT_SECS: u32 = 3 * 60;
/// How long the id of a removed game is remembered, so its link says it's gone
const ENDED_GAME_MEMORY: Duration = Duration::from_secs(24 * 60 * 60);

/// How long a room stays once nobody is in it
#[derive(Clone, Copy, Debug)]
struct RoomTimeouts {
    room: Duration,
    game: Duration,
}

fn env_secs(name: &str, default: u64) -> Duration {
    Duration::from_secs(
        env::var(name)
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(default),
    )
}

/// New chat session is created
#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub id: String,
    pub name: String,
    pub connection: Connection,
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: String,
    /// Socket that closed, ignored when the user already reconnected on another one
    pub connection_id: u64,
}

/// A message from a user's socket, or from the HTTP API on their behalf
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientRequest {
    pub id: String,
    pub msg: ClientMsg,
}

/// `ChessServer` manages chat rooms and responsible for coordinating chat session.
///
/// Implementation is very naïve.
#[derive(Debug)]
pub struct ChessServer {
    sessions: HashMap<String, User>,
    rooms: HashMap<String, Room>,
    /// Ids of removed game rooms, and when they were removed
    ended_games: HashMap<String, Instant>,
}

#[derive(Clone, Debug)]
pub struct User {
    pub id: String,
    pub name: String,
    pub connection: Connection,
    /// Empty until the user's first join
    pub current_room: String,
    pub disconnected_at: Option<Instant>,
}

impl User {
    pub fn new(id: String, name: String, connection: Connection) -> Self {
        Self {
            id,
            name,
            connection,
            current_room: String::new(),
            disconnected_at: None,
        }
    }

    pub fn info(&self) -> UserInfo {
        UserInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            presence: if self.disconnected_at.is_some() {
                Presence::Away
            } else {
                Presence::Online
            },
        }
    }
}

fn side_color(side: Side) -> Color {
    match side {
        Side::White => Color::Light,
        Side::Black => Color::Dark,
    }
}

fn side_name(side: Side) -> &'static str {
    match side {
        Side::White => "White",
        Side::Black => "Black",
    }
}

/// The FEN fields that make two positions the same for the repetition rule
fn position_key(fen: &str) -> String {
    fen.split(' ').take(4).collect::<Vec<_>>().join(" ")
}

/// A chess clock. Only the running side's time changes, so the other side's
/// is exact and the running one is computed from `turn_started`.
#[derive(Clone, Debug)]
struct Clock {
    white_ms: i64,
    black_ms: i64,
    increment_ms: i64,
    running: Option<Side>,
    turn_started: Instant,
    /// Counts turns, so a flag check scheduled for an earlier one can tell
    turn: u64,
}

impl Clock {
    fn new(time_control: TimeControl, now: Instant) -> Self {
        let initial_ms = time_control.initial_secs as i64 * 1000;
        Self {
            white_ms: initial_ms,
            black_ms: initial_ms,
            increment_ms: time_control.increment_secs as i64 * 1000,
            running: None,
            turn_started: now,
            turn: 0,
        }
    }

    fn base_ms(&mut self, side: Side) -> &mut i64 {
        match side {
            Side::White => &mut self.white_ms,
            Side::Black => &mut self.black_ms,
        }
    }

    /// Time `side` has left at `now`, negative once it ran out
    fn remaining_ms(&self, side: Side, now: Instant) -> i64 {
        let base_ms = match side {
            Side::White => self.white_ms,
            Side::Black => self.black_ms,
        };
        if self.running == Some(side) {
            base_ms - now.saturating_duration_since(self.turn_started).as_millis() as i64
        } else {
            base_ms
        }
    }

    /// Ends `mover`'s turn: charges the time it took plus the increment, and
    /// starts the opponent's time. The first move of the game is free.
    fn press(&mut self, mover: Side, now: Instant) {
        if self.running == Some(mover) {
            let left_ms = self.remaining_ms(mover, now);
            *self.base_ms(mover) = left_ms + self.increment_ms;
        }
        self.running = Some(!mover);
        self.turn_started = now;
        self.turn += 1;
    }

    fn stop(&mut self, now: Instant) {
        if let Some(side) = self.running {
            let left_ms = self.remaining_ms(side, now).max(0);
            *self.base_ms(side) = left_ms;
        }
        self.running = None;
    }

    /// The side whose time ran out
    fn flagged(&self, now: Instant) -> Option<Side> {
        self.running
            .filter(|side| self.remaining_ms(*side, now) <= 0)
    }

    fn info(&self, now: Instant) -> ClockInfo {
        ClockInfo {
            white_ms: self.remaining_ms(Side::White, now).max(0),
            black_ms: self.remaining_ms(Side::Black, now).max(0),
            running: self.running,
        }
    }
}

/// A game between two seated players, on a room's board
#[derive(Debug)]
struct Game {
    white: Option<String>,
    black: Option<String>,
    time_control: Option<TimeControl>,
    clock: Option<Clock>,
    result: Option<GameResult>,
    draw_offer: Option<Side>,
    rematch_offer: Option<Side>,
}

impl Game {
    fn new(time_control: Option<TimeControl>, now: Instant) -> Self {
        Self {
            white: None,
            black: None,
            time_control,
            clock: time_control.map(|time_control| Clock::new(time_control, now)),
            result: None,
            draw_offer: None,
            rematch_offer: None,
        }
    }

    fn seat_of(&self, user_id: &str) -> Option<Side> {
        if self.white.as_deref() == Some(user_id) {
            Some(Side::White)
        } else if self.black.as_deref() == Some(user_id) {
            Some(Side::Black)
        } else {
            None
        }
    }

    fn seat_mut(&mut self, side: Side) -> &mut Option<String> {
        match side {
            Side::White => &mut self.white,
            Side::Black => &mut self.black,
        }
    }

    fn is_full(&self) -> bool {
        self.white.is_some() && self.black.is_some()
    }

    fn finish(&mut self, result: GameResult, now: Instant) {
        self.result = Some(result);
        self.draw_offer = None;
        self.rematch_offer = None;
        if let Some(clock) = &mut self.clock {
            clock.stop(now);
        }
    }

    fn info(&self, now: Instant) -> GameInfo {
        GameInfo {
            white: self.white.clone(),
            black: self.black.clone(),
            time_control: self.time_control,
            clock: self.clock.as_ref().map(|clock| clock.info(now)),
            result: self.result,
            draw_offer: self.draw_offer,
            rematch_offer: self.rematch_offer,
        }
    }
}

/// A move played on a room's board, and what to broadcast for it
#[derive(Debug)]
struct PlayedMove {
    message: ServerMsg,
    /// The board after the move, for the player who made it
    position: ServerMsg,
    /// Side effects the clients don't apply themselves: the pawn taken en
    /// passant, the castling rook
    follow_ups: Vec<ServerMsg>,
    /// A promotion swaps the pawn, so clients get the whole board once they're
    /// done animating the move
    resync: bool,
    checkmate: bool,
}

/// Why a move in a game was refused
// the board error is only read through `Debug`, in logs
#[allow(dead_code)]
#[derive(Debug)]
enum MoveRefusal {
    Board(ChessBoardError),
    Spectator,
    WaitingForOpponent,
    NotYourTurn,
    GameOver,
}

impl MoveRefusal {
    /// What to tell the player, nothing when the board resync says it all
    fn message(&self) -> Option<&'static str> {
        match self {
            MoveRefusal::Board(_) => None,
            MoveRefusal::Spectator => Some("You are watching this game"),
            MoveRefusal::WaitingForOpponent => Some("Waiting for an opponent to join"),
            MoveRefusal::NotYourTurn => Some("It's not your turn"),
            MoveRefusal::GameOver => Some("The game is over"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum GameAction {
    Updated,
    RematchStarted,
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
    /// None for a free board, where anyone moves anything
    game: Option<Game>,
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
    /// A free board room
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
            game: None,
        })
    }

    /// A game room, from the initial position with the rules enforced
    fn new_game(name: &str, time_control: Option<TimeControl>, now: Instant) -> Self {
        let chess_board =
            build_board(DEFAULT_FEN, "", true, true).expect("the initial position is valid");

        Self {
            name: name.to_string(),
            chess_board,
            history: vec![(DEFAULT_FEN.to_string(), String::new())],
            history_index: 0,
            sessions: HashSet::new(),
            empty_at: Some(now),
            game: Some(Game::new(time_control, now)),
        }
    }

    fn sync_board_message(&self) -> ServerMsg {
        ServerMsg::SyncBoard {
            room: self.name.clone(),
            fen: self.chess_board.fen.clone(),
            trash: self.chess_board.trash_string(),
        }
    }

    fn options_message(&self) -> ServerMsg {
        ServerMsg::SyncOptions {
            validation: self.chess_board.validation,
            sync: self.chess_board.sync,
        }
    }

    fn game_message(&self, now: Instant) -> ServerMsg {
        ServerMsg::Game {
            game: self.game.as_ref().map(|game| game.info(now)),
        }
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

    /// Plays a move on the board. `Ok(None)` when the stone didn't move.
    fn play_move(
        &mut self,
        piece: &str,
        from: &str,
        to: &str,
        promotion: PromotionKind,
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

        let board_move =
            self.chess_board
                .move_piece_promoting(piece, from_position, to_position, promotion)?;

        let (rook, opponent_pawn, back_rank) = match stone.color() {
            Color::Light => ("lr", "dp", 7),
            Color::Dark => ("dr", "lp", 0),
        };
        let (fen, trash) = (self.chess_board.fen.clone(), self.chess_board.trash_string());
        let move_message = |piece: &str, from: String, to: String| ServerMsg::Move {
            piece: piece.to_string(),
            from,
            to,
            fen: fen.clone(),
            trash: trash.clone(),
        };
        let mut follow_ups = Vec::new();
        let mut resync = false;

        match board_move {
            BoardMove::Passant(captured) => {
                follow_ups.push(move_message(
                    opponent_pawn,
                    captured.to_string(),
                    DELETED.to_string(),
                ));
            }
            BoardMove::Castle(side) => {
                let (rook_from, rook_to) = match side {
                    CastlePosition::KingSide => (7, 5),
                    CastlePosition::QueenSide => (0, 3),
                };
                follow_ups.push(move_message(
                    rook,
                    Position::new(rook_from, back_rank).to_string(),
                    Position::new(rook_to, back_rank).to_string(),
                ));
            }
            BoardMove::Promotion(_) => resync = true,
            BoardMove::Normal => {}
        }

        let message = move_message(piece, from.to_string(), to.to_string());
        self.history.truncate(self.history_index + 1);
        self.history.push((fen.clone(), trash.clone()));
        self.history_index += 1;

        Ok(Some(PlayedMove {
            message,
            position: ServerMsg::Position { fen, trash },
            follow_ups,
            resync,
            checkmate: self.chess_board.is_checkmate(),
        }))
    }

    /// Plays `user_id`'s move: anyone's on a free board, only the seated
    /// player whose turn it is in a game
    fn play(
        &mut self,
        user_id: &str,
        piece: &str,
        from: &str,
        to: &str,
        promotion: PromotionKind,
        now: Instant,
    ) -> Result<Option<PlayedMove>, MoveRefusal> {
        let Some(game) = &self.game else {
            return self
                .play_move(piece, from, to, promotion)
                .map_err(MoveRefusal::Board);
        };

        if game.result.is_some() {
            return Err(MoveRefusal::GameOver);
        }
        let Some(seat) = game.seat_of(user_id) else {
            return Err(MoveRefusal::Spectator);
        };
        if !game.is_full() {
            return Err(MoveRefusal::WaitingForOpponent);
        }
        if seat != self.chess_board.turn {
            return Err(MoveRefusal::NotYourTurn);
        }
        // the move arrived after the player's time ran out
        if self.check_flag(now) {
            return Err(MoveRefusal::GameOver);
        }

        let Some(played) = self
            .play_move(piece, from, to, promotion)
            .map_err(MoveRefusal::Board)?
        else {
            return Ok(None);
        };

        if let Some(game) = &mut self.game {
            if let Some(clock) = &mut game.clock {
                clock.press(seat, now);
            }
            // moving declines the opponent's draw offer
            if game.draw_offer == Some(!seat) {
                game.draw_offer = None;
            }
        }
        self.check_ending(seat, now);

        Ok(Some(played))
    }

    fn repetitions(&self) -> usize {
        let current = position_key(&self.chess_board.fen);
        self.history[..=self.history_index]
            .iter()
            .filter(|(fen, _)| position_key(fen) == current)
            .count()
    }

    /// Ends the game if `mover`'s move finished it
    fn check_ending(&mut self, mover: Side, now: Instant) {
        let board = &self.chess_board;
        let result = if board.is_checkmate() {
            Some(GameResult::Checkmate { winner: mover })
        } else if board.is_stalemate() {
            Some(GameResult::Stalemate)
        } else if board.has_insufficient_material() {
            Some(GameResult::InsufficientMaterial)
        } else if board.half_move_clock >= 100 {
            Some(GameResult::FiftyMoves)
        } else if self.repetitions() >= 3 {
            Some(GameResult::Repetition)
        } else {
            None
        };

        if let (Some(result), Some(game)) = (result, &mut self.game) {
            game.finish(result, now);
        }
    }

    /// Ends the game if the running side is out of time. Returns whether it did.
    fn check_flag(&mut self, now: Instant) -> bool {
        let Some(game) = &mut self.game else {
            return false;
        };
        if game.result.is_some() {
            return false;
        }
        let Some(flagged) = game.clock.as_ref().and_then(|clock| clock.flagged(now)) else {
            return false;
        };

        let winner = !flagged;
        let result = if self.chess_board.has_material_beyond_king(side_color(winner)) {
            GameResult::Timeout { winner }
        } else {
            GameResult::TimeoutVsInsufficientMaterial
        };
        game.finish(result, now);
        true
    }

    /// The user's seat, taking a free one if the game hasn't ended
    fn take_seat(&mut self, user_id: &str) -> Option<Side> {
        let game = self.game.as_mut()?;
        if let Some(side) = game.seat_of(user_id) {
            return Some(side);
        }
        if game.result.is_some() {
            return None;
        }

        let side = [Side::White, Side::Black]
            .into_iter()
            .find(|side| game.seat_mut(*side).is_none())?;
        *game.seat_mut(side) = Some(user_id.to_string());
        Some(side)
    }

    /// Resignations, draw and rematch offers. The error is what to tell the user.
    fn game_action(
        &mut self,
        user_id: &str,
        msg: &ClientMsg,
        now: Instant,
    ) -> Result<GameAction, &'static str> {
        let Some(game) = &mut self.game else {
            return Err("There is no game on a free board");
        };
        let Some(seat) = game.seat_of(user_id) else {
            return Err("Only the players can do that");
        };
        let game_over = game.result.is_some();

        match msg {
            ClientMsg::Resign if !game_over => {
                game.finish(GameResult::Resignation { winner: !seat }, now)
            }
            ClientMsg::OfferDraw | ClientMsg::AcceptDraw
                if !game_over && game.draw_offer == Some(!seat) =>
            {
                game.finish(GameResult::Agreement, now)
            }
            ClientMsg::OfferDraw if !game_over && game.is_full() => game.draw_offer = Some(seat),
            ClientMsg::OfferDraw if !game_over => return Err("Waiting for an opponent to join"),
            ClientMsg::AcceptDraw if !game_over => return Err("There is no draw offer"),
            ClientMsg::DeclineDraw if game.draw_offer == Some(!seat) => game.draw_offer = None,
            ClientMsg::DeclineDraw => {}
            ClientMsg::Rematch if game_over && game.rematch_offer == Some(!seat) => {
                self.start_rematch(now);
                return Ok(GameAction::RematchStarted);
            }
            ClientMsg::Rematch if game_over => game.rematch_offer = Some(seat),
            ClientMsg::Rematch => return Err("The game is still going"),
            _ => return Err("The game is over"),
        }

        Ok(GameAction::Updated)
    }

    /// Same players with the colors swapped, from the initial position
    fn start_rematch(&mut self, now: Instant) {
        if let Some(game) = &mut self.game {
            let (white, black) = (game.black.take(), game.white.take());
            *game = Game {
                white,
                black,
                ..Game::new(game.time_control, now)
            };
        }

        self.history.truncate(1);
        self.load_history(0)
            .expect("a game's initial position is valid");
    }
}

impl ChessServer {
    pub fn new() -> ChessServer {
        // default room
        let mut rooms = HashMap::new();
        rooms.insert(
            MAIN_ROOM.to_owned(),
            Room::new(MAIN_ROOM, None, None).expect("Failed to create default room"),
        );

        ChessServer {
            sessions: HashMap::new(),
            rooms,
            ended_games: HashMap::new(),
        }
    }
}

fn to_json(message: &ServerMsg) -> String {
    serde_json::to_string(message).expect("server messages serialize")
}

impl ChessServer {
    /// Send message to all users in the room
    fn send_message(&self, room_name: &str, message: &ServerMsg, skip_id: Option<&str>) {
        let Some(room) = self.rooms.get(room_name) else {
            return;
        };
        let text = to_json(message);

        for id in &room.sessions {
            if Some(id.as_str()) != skip_id {
                self.send_text(id, &text);
            }
        }
    }

    fn send_message_to_session(&self, id: &str, message: &ServerMsg) {
        self.send_text(id, &to_json(message));
    }

    fn send_text(&self, id: &str, text: &str) {
        if let Some(user) = self.sessions.get(id) {
            user.connection.send(Outbound::Text(text.to_owned()));
        }
    }

    fn notify(&self, id: &str, level: NotifyType, message: &str) {
        self.send_message_to_session(id, &ServerMsg::notify(level, message));
    }

    /// Name of the room the user is in
    fn room_of(&self, id: &str) -> Option<String> {
        let room_name = self.sessions.get(id).map(|user| user.current_room.clone());
        if room_name.is_none() {
            log::error!("No user found for id {}", id);
        }
        room_name
    }

    fn users_message(&self, room: &Room) -> ServerMsg {
        ServerMsg::SyncUsers {
            room: room.name.clone(),
            users: room
                .sessions
                .iter()
                .filter_map(|id| self.sessions.get(id))
                .map(User::info)
                .collect(),
        }
    }

    /// Sends a session everything it needs to render its room
    fn sync_session(&self, id: &str, room_name: &str) {
        let Some(room) = self.rooms.get(room_name) else {
            log::error!("No room found with name {}", room_name);
            return;
        };

        self.send_message_to_session(id, &room.sync_board_message());
        self.send_message_to_session(id, &self.users_message(room));
        self.send_message_to_session(id, &room.options_message());
        self.send_message_to_session(id, &room.game_message(Instant::now()));
        if room.game.is_none() && room.chess_board.is_checkmate() {
            self.send_message_to_session(id, &ServerMsg::Checkmate);
        }
    }

    /// Sends the room's board to all its users
    fn broadcast_board(&self, room_name: &str) {
        let Some(room) = self.rooms.get(room_name) else {
            return;
        };

        self.send_message(room_name, &room.sync_board_message(), None);
        if room.game.is_none() && room.chess_board.is_checkmate() {
            self.send_message(room_name, &ServerMsg::Checkmate, None);
        }
    }

    fn broadcast_game(&self, room_name: &str, now: Instant) {
        if let Some(room) = self.rooms.get(room_name) {
            self.send_message(room_name, &room.game_message(now), None);
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

        if let Some(room) = self.rooms.get_mut(&room_name) {
            room.sessions.remove(id);

            if room.sessions.is_empty() {
                room.empty_at = Some(Instant::now());
            }
        }

        self.send_message(
            &room_name,
            &ServerMsg::UserLeft { id: id.to_string() },
            None,
        );
    }

    fn remove_session(&mut self, id: &str) {
        self.leave_room(id);
        self.sessions.remove(id);
    }

    fn remove_expired_rooms(&mut self, now: Instant, timeouts: RoomTimeouts) {
        let mut ended_games = Vec::new();
        self.rooms.retain(|name, room| {
            let timeout = if room.game.is_some() {
                timeouts.game
            } else {
                timeouts.room
            };
            let expired = room
                .empty_at
                .is_some_and(|empty_at| now.saturating_duration_since(empty_at) > timeout);
            if expired {
                log::info!("Room {} is empty, removing", name);
                if room.game.is_some() {
                    ended_games.push(name.clone());
                }
            }
            !expired
        });

        self.ended_games.extend(ended_games.into_iter().map(|name| (name, now)));
        self.ended_games
            .retain(|_, ended_at| now.saturating_duration_since(*ended_at) < ENDED_GAME_MEMORY);
    }

    fn new_room_id(&self) -> String {
        loop {
            let id = uuid::Uuid::new_v4().simple().to_string()[..8].to_string();
            if !self.rooms.contains_key(&id) && !self.ended_games.contains_key(&id) {
                return id;
            }
        }
    }

    fn join(&mut self, id: &str, room: &str, fen: Option<String>, trash: Option<String>) {
        let Some(room_name) = sanitize_room_name(room) else {
            self.notify(id, NotifyType::Error, "Invalid room name");
            return;
        };
        // an old game link would otherwise open a new free board
        if !self.rooms.contains_key(&room_name) && self.ended_games.contains_key(&room_name) {
            self.send_message_to_session(id, &ServerMsg::RoomGone { room: room_name });
            return;
        }

        // Create the room before leaving the current one, so a bad FEN leaves
        // the user where they were
        if !self.rooms.contains_key(&room_name) {
            let Ok(room) = Room::new(&room_name, fen, trash) else {
                self.notify(id, NotifyType::Error, "Failed to create room");
                return;
            };
            self.rooms.insert(room_name.clone(), room);
        }

        // Joining the current room again, e.g. after a reconnect, only syncs it
        let already_in = self
            .sessions
            .get(id)
            .is_some_and(|user| user.current_room == room_name);
        if !already_in {
            self.leave_room(id);
            self.enter_room(id, &room_name);
        }

        let (was_seated, seat) = match self.rooms.get_mut(&room_name) {
            Some(room) => {
                let was_seated = room
                    .game
                    .as_ref()
                    .is_some_and(|game| game.seat_of(id).is_some());
                (was_seated, room.take_seat(id))
            }
            None => (false, None),
        };
        let is_game = self
            .rooms
            .get(&room_name)
            .is_some_and(|room| room.game.is_some());

        self.sync_session(id, &room_name);
        if seat.is_some() && !was_seated {
            self.broadcast_game(&room_name, Instant::now());
        }

        if !already_in {
            let message = match (is_game, seat) {
                (true, Some(side)) => format!("Joined the game as {}", side_name(side)),
                (true, None) => "Watching the game".to_string(),
                (false, _) => format!("Joined room {}", room_name),
            };
            self.notify(id, NotifyType::Success, &message);

            // notify all users in room
            if let Some(user) = self.sessions.get(id) {
                let message = ServerMsg::UserJoined { user: user.info() };
                self.send_message(&room_name, &message, Some(id));
            }
        }
    }

    fn rename(&mut self, id: &str, name: &str) {
        let Some(name) = sanitize_username(name) else {
            self.notify(id, NotifyType::Error, "A name is required");
            return;
        };
        let Some(user) = self.sessions.get_mut(id) else {
            return;
        };

        user.name = name;
        let room_name = user.current_room.clone();
        let message = ServerMsg::UserJoined { user: user.info() };

        // notify all users in room
        self.send_message(&room_name, &message, None);
    }

    fn chat(&self, id: &str, text: &str) {
        let Some(user) = self.sessions.get(id) else {
            return;
        };
        let message = ServerMsg::Chat {
            name: user.name.clone(),
            text: text.to_string(),
        };

        self.send_message(&user.current_room, &message, Some(id));
    }

    fn play(
        &mut self,
        id: &str,
        piece: &str,
        from: &str,
        to: &str,
        promotion: PromotionKind,
        ctx: &mut Context<Self>,
    ) {
        let Some(room_name) = self.room_of(id) else {
            return;
        };
        let Some(room) = self.rooms.get_mut(&room_name) else {
            return;
        };
        let now = Instant::now();
        let is_game = room.game.is_some();

        match room.play(id, piece, from, to, promotion, now) {
            Ok(Some(played)) => {
                self.send_message(&room_name, &played.message, Some(id));
                for follow_up in &played.follow_ups {
                    self.send_message(&room_name, follow_up, None);
                }
                self.send_message_to_session(id, &played.position);
                if is_game {
                    self.broadcast_game(&room_name, now);
                    self.schedule_flag_check(&room_name, ctx, now);
                } else if played.checkmate && !played.resync {
                    self.send_message(&room_name, &ServerMsg::Checkmate, None);
                }
                if played.resync {
                    ctx.run_later(PROMOTION_SYNC_DELAY, move |act, _| {
                        act.broadcast_board(&room_name)
                    });
                }
            }
            Ok(None) => {}
            Err(refusal) => {
                log::warn!(
                    "Room: {} -> failed attempt to move piece {} from {} to {} -> {:?}",
                    room_name,
                    piece,
                    from,
                    to,
                    refusal
                );
                let sync_board = room.sync_board_message();
                self.send_message_to_session(id, &sync_board);
                if let Some(message) = refusal.message() {
                    self.notify(id, NotifyType::Warning, message);
                }
                // the move may have arrived after the clock flagged
                if matches!(refusal, MoveRefusal::GameOver) {
                    self.broadcast_game(&room_name, now);
                }
            }
        }
    }

    /// Checks the running side's clock once its time should be up
    fn schedule_flag_check(&self, room_name: &str, ctx: &mut Context<Self>, now: Instant) {
        let Some(clock) = self
            .rooms
            .get(room_name)
            .and_then(|room| room.game.as_ref())
            .and_then(|game| game.clock.as_ref())
        else {
            return;
        };
        let Some(running) = clock.running else {
            return;
        };

        let delay = Duration::from_millis(clock.remaining_ms(running, now).max(0) as u64 + 50);
        let turn = clock.turn;
        let room_name = room_name.to_string();
        ctx.run_later(delay, move |act, ctx| act.flag_check(&room_name, turn, ctx));
    }

    fn flag_check(&mut self, room_name: &str, turn: u64, ctx: &mut Context<Self>) {
        let Some(room) = self.rooms.get_mut(room_name) else {
            return;
        };
        let now = Instant::now();
        let still_same_turn = room
            .game
            .as_ref()
            .and_then(|game| game.clock.as_ref())
            .is_some_and(|clock| clock.turn == turn);
        // a later move scheduled its own check
        if !still_same_turn {
            return;
        }

        if room.check_flag(now) {
            self.broadcast_game(room_name, now);
        } else {
            // timers can fire a little early
            self.schedule_flag_check(room_name, ctx, now);
        }
    }

    /// Reset, undo, redo and options, which only free boards allow
    fn board_command(&mut self, id: &str, msg: ClientMsg) {
        let Some(room_name) = self.room_of(id) else {
            return;
        };
        let Some(room) = self.rooms.get_mut(&room_name) else {
            return;
        };
        if room.game.is_some() {
            self.notify(id, NotifyType::Warning, "Not available during a game");
            return;
        }

        match msg {
            ClientMsg::Reset => match room.reset() {
                Ok(()) => self.broadcast_board(&room_name),
                Err(_) => self.notify(id, NotifyType::Error, "Failed to reset board"),
            },
            ClientMsg::Undo | ClientMsg::Redo => {
                let (step, action): (fn(&mut Room) -> Result<bool, ChessBoardError>, _) =
                    match msg {
                        ClientMsg::Undo => (Room::undo, "undo"),
                        _ => (Room::redo, "redo"),
                    };
                match step(room) {
                    Ok(true) => self.broadcast_board(&room_name),
                    Ok(false) => self.notify(
                        id,
                        NotifyType::Warning,
                        &format!("No more moves to {}", action),
                    ),
                    Err(_) => {
                        self.notify(id, NotifyType::Error, &format!("Failed to {} move", action))
                    }
                }
            }
            ClientMsg::Options { validation, sync } => {
                let (level, notification) = match room.set_options(validation, sync) {
                    Ok(()) => (NotifyType::Success, "Options applied"),
                    Err(_) => (NotifyType::Error, "Failed to apply options"),
                };
                let options = room.options_message();

                self.notify(id, level, notification);
                self.send_message(&room_name, &options, None);
            }
            _ => {}
        }
    }

    fn new_game(&mut self, id: &str, color: SeatChoice, time_control: Option<TimeControl>) {
        if let Some(time_control) = time_control {
            if time_control.initial_secs == 0
                || time_control.initial_secs > MAX_INITIAL_SECS
                || time_control.increment_secs > MAX_INCREMENT_SECS
            {
                self.notify(id, NotifyType::Error, "Invalid time control");
                return;
            }
        }

        let side = match color {
            SeatChoice::White => Side::White,
            SeatChoice::Black => Side::Black,
            SeatChoice::Random if uuid::Uuid::new_v4().as_bytes()[0] & 1 == 0 => Side::White,
            SeatChoice::Random => Side::Black,
        };
        let room_id = self.new_room_id();
        let mut room = Room::new_game(&room_id, time_control, Instant::now());
        if let Some(game) = &mut room.game {
            *game.seat_mut(side) = Some(id.to_string());
        }
        self.rooms.insert(room_id.clone(), room);

        self.send_message_to_session(id, &ServerMsg::GameCreated { room: room_id });
    }

    fn game_action(&mut self, id: &str, msg: &ClientMsg) {
        let Some(room_name) = self.room_of(id) else {
            return;
        };
        let Some(room) = self.rooms.get_mut(&room_name) else {
            return;
        };
        let now = Instant::now();

        match room.game_action(id, msg, now) {
            Ok(action) => {
                if action == GameAction::RematchStarted {
                    self.broadcast_board(&room_name);
                }
                self.broadcast_game(&room_name, now);
            }
            Err(message) => self.notify(id, NotifyType::Warning, message),
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

        let timeouts = RoomTimeouts {
            // 5 minutes
            room: env_secs("ROOM_TIMEOUT", 300),
            // a game's final position stays an hour, for the players to look at
            game: env_secs("GAME_ROOM_TIMEOUT", 3600),
        };
        // 5 minutes
        let user_timeout = env_secs("USER_TIMEOUT", 300);

        ctx.run_interval(Duration::from_secs(5), move |act, _| {
            let now = Instant::now();
            act.remove_expired_rooms(now, timeouts);

            let expired_sessions: Vec<String> = act
                .sessions
                .values()
                .filter(|user| {
                    user.disconnected_at.is_some_and(|disconnected_at| {
                        now.saturating_duration_since(disconnected_at) > user_timeout
                    })
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
/// Register new session and assign unique id to this session. The client
/// joins its room right after connecting.
impl Handler<Connect> for ChessServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) {
        log::debug!("Someone joined");

        let Connect {
            connection,
            id,
            name,
        } = msg;

        if let Some(user) = self.sessions.get_mut(&id) {
            // Same user on a new socket (page reload or another tab)
            user.connection
                .send(Outbound::Text(to_json(&ServerMsg::Replaced)));
            user.connection.send(Outbound::Kick);
            user.connection = connection;
            user.disconnected_at = None;

            let room_name = user.current_room.clone();
            let message = ServerMsg::UserPresence {
                id: id.clone(),
                presence: Presence::Online,
            };
            self.send_message(&room_name, &message, Some(&id));
        } else {
            self.sessions
                .insert(id.clone(), User::new(id, name, connection));
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

        if user.connection.id != msg.connection_id {
            return;
        }

        user.disconnected_at = Some(Instant::now());
        let room_name = user.current_room.clone();
        let message = ServerMsg::UserPresence {
            id: msg.id.clone(),
            presence: Presence::Away,
        };

        self.send_message(&room_name, &message, Some(&msg.id));
    }
}

impl Handler<ClientRequest> for ChessServer {
    type Result = ();

    fn handle(&mut self, request: ClientRequest, ctx: &mut Context<Self>) {
        let ClientRequest { id, msg } = request;
        if !self.sessions.contains_key(&id) {
            log::error!("No user found for id {}", id);
            return;
        }

        match msg {
            ClientMsg::Join { room, fen, trash } => self.join(&id, &room, fen, trash),
            ClientMsg::Username { name } => self.rename(&id, &name),
            ClientMsg::Chat { text } => self.chat(&id, &text),
            ClientMsg::Move {
                piece,
                from,
                to,
                promotion,
            } => self.play(
                &id,
                &piece,
                &from,
                &to,
                promotion.unwrap_or_default(),
                ctx,
            ),
            msg @ (ClientMsg::Reset
            | ClientMsg::Undo
            | ClientMsg::Redo
            | ClientMsg::Options { .. }) => self.board_command(&id, msg),
            ClientMsg::NewGame {
                color,
                time_control,
            } => self.new_game(&id, color, time_control),
            msg @ (ClientMsg::Resign
            | ClientMsg::OfferDraw
            | ClientMsg::AcceptDraw
            | ClientMsg::DeclineDraw
            | ClientMsg::Rematch) => self.game_action(&id, &msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn play(room: &mut Room, piece: &str, from: &str, to: &str) -> Option<PlayedMove> {
        room.play_move(piece, from, to, PromotionKind::Queen)
            .unwrap()
    }

    /// The piece, squares and side to move of a move message
    fn move_parts(msg: &ServerMsg) -> (&str, &str, &str, &str) {
        match msg {
            ServerMsg::Move {
                piece,
                from,
                to,
                fen,
                ..
            } => (piece, from, to, fen.split(' ').nth(1).unwrap()),
            other => panic!("not a move: {other:?}"),
        }
    }

    #[test]
    fn test_play_move_rejects_bad_input() {
        let mut room = Room::new("test", None, None).unwrap();
        let fen = room.chess_board.fen.clone();

        for (piece, from, to) in [
            ("xx", "e2", "e4"),
            ("lp", "i1", "e4"),
            ("lp", "e2", "z9"),
            ("lq", "e2", "e4"),
        ] {
            assert!(room
                .play_move(piece, from, to, PromotionKind::Queen)
                .is_err());
        }
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

        // the follow ups carry the board after the whole move, black to move
        let played = play(&mut room, "lp", "e5", "d6").unwrap();
        assert_eq!(played.follow_ups.len(), 1);
        assert_eq!(move_parts(&played.follow_ups[0]), ("dp", "d5", DELETED, "b"));
        assert_eq!(move_parts(&played.message), ("lp", "e5", "d6", "b"));
        let ServerMsg::Position { fen, .. } = &played.position else {
            panic!("not a position");
        };
        assert_eq!(fen, &room.chess_board.fen);
        assert!(fen.starts_with("r3k2r/8/3P4/8/"));

        let played = play(&mut room, "dk", "e8", "c8").unwrap();
        assert_eq!(played.follow_ups.len(), 1);
        assert_eq!(move_parts(&played.follow_ups[0]), ("dr", "a8", "d8", "w"));

        let mut room = validated_room("8/4P3/8/8/8/8/8/k6K w - - 0 1");
        let played = room
            .play_move("lp", "e7", "e8", PromotionKind::Knight)
            .unwrap()
            .unwrap();
        assert!(played.resync);
        assert!(room.chess_board.fen.starts_with("4N3/"));

        let mut room = validated_room("7k/8/6K1/8/8/8/8/5Q2 w - - 0 1");
        let played = play(&mut room, "lq", "f1", "f8").unwrap();
        assert!(played.checkmate);
    }

    fn time_control(initial_secs: u32, increment_secs: u32) -> Option<TimeControl> {
        Some(TimeControl {
            initial_secs,
            increment_secs,
        })
    }

    fn ms(millis: u64) -> Duration {
        Duration::from_millis(millis)
    }

    #[test]
    fn test_clock() {
        let start = Instant::now();
        let mut clock = Clock::new(time_control(60, 2).unwrap(), start);

        // the first move is free and starts black's time
        clock.press(Side::White, start + ms(5_000));
        assert_eq!(clock.info(start + ms(5_000)).white_ms, 60_000);
        assert_eq!(clock.running, Some(Side::Black));

        // black takes 10s and gets 2s back
        clock.press(Side::Black, start + ms(15_000));
        let info = clock.info(start + ms(15_000));
        assert_eq!((info.white_ms, info.black_ms), (60_000, 52_000));

        assert_eq!(clock.flagged(start + ms(74_999)), None);
        assert_eq!(clock.flagged(start + ms(75_000)), Some(Side::White));

        clock.stop(start + ms(20_000));
        assert_eq!(clock.info(start + ms(90_000)).white_ms, 55_000);
        assert_eq!(clock.running, None);
    }

    fn game_room(time_control: Option<TimeControl>, start: Instant) -> Room {
        let mut room = Room::new_game("game", time_control, start);
        assert_eq!(room.take_seat("w"), Some(Side::White));
        assert_eq!(room.take_seat("b"), Some(Side::Black));
        room
    }

    fn game_move(room: &mut Room, user: &str, piece: &str, from: &str, to: &str, now: Instant) {
        room.play(user, piece, from, to, PromotionKind::Queen, now)
            .unwrap()
            .unwrap();
    }

    #[test]
    fn test_game_seats_and_turns() {
        let start = Instant::now();
        let mut room = Room::new_game("game", None, start);
        assert_eq!(room.take_seat("w"), Some(Side::White));

        let refused =
            |room: &mut Room, user: &str| room.play(user, "lp", "e2", "e4", PromotionKind::Queen, start);

        assert!(matches!(refused(&mut room, "w"), Err(MoveRefusal::WaitingForOpponent)));
        assert_eq!(room.take_seat("b"), Some(Side::Black));
        assert_eq!(room.take_seat("w"), Some(Side::White));
        assert_eq!(room.take_seat("spectator"), None);

        assert!(matches!(refused(&mut room, "spectator"), Err(MoveRefusal::Spectator)));
        assert!(matches!(refused(&mut room, "b"), Err(MoveRefusal::NotYourTurn)));
        // white can't move black's pieces either
        assert!(matches!(
            room.play("w", "dp", "e7", "e5", PromotionKind::Queen, start),
            Err(MoveRefusal::Board(_))
        ));

        game_move(&mut room, "w", "lp", "e2", "e4", start);
        game_move(&mut room, "b", "dp", "e7", "e5", start);
    }

    #[test]
    fn test_game_endings() {
        let start = Instant::now();
        let mut room = game_room(None, start);

        // fool's mate
        game_move(&mut room, "w", "lp", "f2", "f3", start);
        game_move(&mut room, "b", "dp", "e7", "e5", start);
        game_move(&mut room, "w", "lp", "g2", "g4", start);
        game_move(&mut room, "b", "dq", "d8", "h4", start);
        let result = room.game.as_ref().unwrap().result;
        assert_eq!(result, Some(GameResult::Checkmate { winner: Side::Black }));
        assert!(matches!(
            room.play("w", "lp", "a2", "a3", PromotionKind::Queen, start),
            Err(MoveRefusal::GameOver)
        ));

        // knights going back and forth repeat the initial position
        let mut room = game_room(None, start);
        for _ in 0..2 {
            game_move(&mut room, "w", "ln", "g1", "f3", start);
            game_move(&mut room, "b", "dn", "g8", "f6", start);
            game_move(&mut room, "w", "ln", "f3", "g1", start);
            game_move(&mut room, "b", "dn", "f6", "g8", start);
        }
        let result = room.game.as_ref().unwrap().result;
        assert_eq!(result, Some(GameResult::Repetition));
    }

    #[test]
    fn test_game_clock_and_flag() {
        let start = Instant::now();
        let mut room = game_room(time_control(60, 0), start);

        game_move(&mut room, "w", "lp", "e2", "e4", start);
        // black thinks for 61s
        assert!(!room.check_flag(start + ms(59_000)));
        assert!(matches!(
            room.play("b", "dp", "e7", "e5", PromotionKind::Queen, start + ms(61_000)),
            Err(MoveRefusal::GameOver)
        ));
        let game = room.game.as_ref().unwrap();
        assert_eq!(game.result, Some(GameResult::Timeout { winner: Side::White }));
        assert_eq!(game.clock.as_ref().unwrap().running, None);

        // a lone king can't win on time
        let mut room = game_room(time_control(60, 0), start);
        room.history[0] = ("q3k3/8/8/8/8/8/8/4K3 w - - 0 1".to_string(), String::new());
        room.load_history(0).unwrap();
        game_move(&mut room, "w", "lk", "e1", "d1", start);
        assert!(room.check_flag(start + ms(61_000)));
        let result = room.game.as_ref().unwrap().result;
        assert_eq!(result, Some(GameResult::TimeoutVsInsufficientMaterial));
    }

    #[test]
    fn test_draw_resign_rematch() {
        let start = Instant::now();
        let mut room = game_room(time_control(300, 3), start);
        let act = |room: &mut Room, user: &str, msg: ClientMsg| room.game_action(user, &msg, start);

        assert!(act(&mut room, "spectator", ClientMsg::Resign).is_err());
        assert_eq!(act(&mut room, "w", ClientMsg::OfferDraw), Ok(GameAction::Updated));
        assert!(act(&mut room, "w", ClientMsg::AcceptDraw).is_err());
        assert_eq!(act(&mut room, "b", ClientMsg::DeclineDraw), Ok(GameAction::Updated));
        assert_eq!(room.game.as_ref().unwrap().draw_offer, None);

        act(&mut room, "b", ClientMsg::OfferDraw).unwrap();
        act(&mut room, "w", ClientMsg::AcceptDraw).unwrap();
        assert_eq!(room.game.as_ref().unwrap().result, Some(GameResult::Agreement));
        assert!(act(&mut room, "w", ClientMsg::Resign).is_err());

        game_move_is_refused_after_the_end(&mut room, start);

        assert_eq!(act(&mut room, "w", ClientMsg::Rematch), Ok(GameAction::Updated));
        assert_eq!(act(&mut room, "b", ClientMsg::Rematch), Ok(GameAction::RematchStarted));
        let game = room.game.as_ref().unwrap();
        assert_eq!(game.white.as_deref(), Some("b"));
        assert_eq!(game.black.as_deref(), Some("w"));
        assert_eq!(game.result, None);
        assert_eq!(game.clock.as_ref().unwrap().info(start).white_ms, 300_000);

        game_move(&mut room, "b", "lp", "e2", "e4", start);
        act(&mut room, "w", ClientMsg::Resign).unwrap();
        let result = room.game.as_ref().unwrap().result;
        assert_eq!(result, Some(GameResult::Resignation { winner: Side::White }));
    }

    #[test]
    fn test_expired_game_link_is_gone() {
        let mut server = ChessServer::new();
        let (connection, mut socket) = Connection::for_test(1);
        server
            .sessions
            .insert("u".to_string(), User::new("u".to_string(), "U".to_string(), connection));

        let start = Instant::now();
        server.rooms.insert("free".to_string(), Room::new("free", None, None).unwrap());
        server
            .rooms
            .insert("game".to_string(), Room::new_game("game", None, start));
        let timeouts = RoomTimeouts {
            room: Duration::from_secs(300),
            game: Duration::from_secs(3600),
        };

        // a free room goes first, the game stays for its players
        server.remove_expired_rooms(start + Duration::from_secs(600), timeouts);
        assert!(!server.rooms.contains_key("free"));
        assert!(server.rooms.contains_key("game"));

        server.remove_expired_rooms(start + Duration::from_secs(4000), timeouts);
        assert!(!server.rooms.contains_key("game"));

        server.join("u", "game", None, None);
        assert!(!server.rooms.contains_key("game"));
        let sent = match socket.try_recv() {
            Ok(Outbound::Text(text)) => serde_json::from_str::<ServerMsg>(&text).unwrap(),
            other => panic!("expected a message, got {other:?}"),
        };
        assert_eq!(sent, ServerMsg::RoomGone { room: "game".to_string() });

        // other names still open a free board
        server.join("u", "free", None, None);
        assert!(server.rooms.contains_key("free"));
    }

    fn game_move_is_refused_after_the_end(room: &mut Room, now: Instant) {
        assert!(matches!(
            room.play("w", "lp", "e2", "e4", PromotionKind::Queen, now),
            Err(MoveRefusal::GameOver)
        ));
    }
}
