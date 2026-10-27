use cfg_if::cfg_if;
use leptos::prelude::*;
use std::collections::BTreeMap;
use web_sys::WebSocket;

use super::{ChessBoard, ChessBoardBuilder};

use crate::entities::{
    notification::{Notification, NotifyType},
    position::Position,
    protocol::{ClientMsg, GameInfo, Side},
    room::RoomStatus,
    stone::Stone,
};
use crate::utils::my_id;

/// A piece picked by a tap, or by the start of a drag
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selection {
    pub piece: String,
    /// None for a piece in the trash
    pub from: Option<Position>,
}

/// A pawn move waiting for the player to pick what it promotes to
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingMove {
    pub piece: String,
    pub from: String,
    pub to: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Connection {
    Connecting,
    Open,
    Closed,
    /// The user opened the board in another tab, which took over
    Replaced,
}

#[derive(Clone, Debug)]
pub struct StonesSignals {
    board_stones: BTreeMap<String, RwSignal<StoneSignal>>,
    deleted_stones_idx: usize,
    deleted_stones: BTreeMap<usize, RwSignal<StoneSignal>>,
}

#[allow(dead_code)]
impl StonesSignals {
    pub fn new() -> Self {
        Self {
            board_stones: BTreeMap::new(),
            deleted_stones: BTreeMap::new(),
            deleted_stones_idx: 0,
        }
    }

    pub fn get_board_stone(&self, key: &str) -> Option<RwSignal<StoneSignal>> {
        self.board_stones.get(key).cloned()
    }

    pub fn get_deleted_stone(&self, idx: &usize) -> Option<RwSignal<StoneSignal>> {
        self.deleted_stones.get(idx).cloned()
    }

    pub fn board_stones(&self) -> &BTreeMap<String, RwSignal<StoneSignal>> {
        &self.board_stones
    }

    pub fn deleted_stones(&self) -> &BTreeMap<usize, RwSignal<StoneSignal>> {
        &self.deleted_stones
    }

    pub fn add_board_stone(&mut self, position: Position, stone: Stone) {
        let stone_signal = StoneSignal::new(Some(position.clone()), stone);
        let key = stone_signal.unique_key();
        let stone_signal = RwSignal::new(stone_signal);

        self.board_stones.insert(key, stone_signal);
    }

    pub fn add_board_stone_signal(&mut self, key: String, stone_signal: RwSignal<StoneSignal>) {
        self.board_stones.insert(key, stone_signal);
    }

    pub fn remove_board_stone(&mut self, key: String) -> Option<RwSignal<StoneSignal>> {
        self.board_stones.remove(&key)
    }

    pub fn add_deleted_stone(&mut self, stone: Stone) {
        let stone_signal = StoneSignal::new_deleted(stone);
        let stone_signal = RwSignal::new(stone_signal);
        self.deleted_stones
            .insert(self.deleted_stones_idx, stone_signal);
        self.deleted_stones_idx += 1;
    }

    pub fn add_deleted_stone_signal(&mut self, stone_signal: RwSignal<StoneSignal>) {
        self.deleted_stones
            .insert(self.deleted_stones_idx, stone_signal);
        self.deleted_stones_idx += 1;
    }

    pub fn remove_deleted_stone(&mut self, idx: usize) -> Option<RwSignal<StoneSignal>> {
        self.deleted_stones.remove(&idx)
    }

    pub fn clear_board_stones(&mut self) {
        self.board_stones.clear();
    }

    pub fn clear_deleted_stones(&mut self) {
        self.deleted_stones.clear();
    }
}

#[derive(Clone, Debug)]
pub struct StoneSignal {
    position: Option<Position>,
    stone: Stone,
    dragging: bool,
    deleted: bool,
}

#[allow(dead_code)]
impl StoneSignal {
    pub fn new(position: Option<Position>, stone: Stone) -> Self {
        Self {
            position,
            stone,
            dragging: false,
            deleted: false,
        }
    }

    pub fn new_deleted(stone: Stone) -> Self {
        Self {
            position: None,
            stone,
            dragging: false,
            deleted: true,
        }
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted
    }

    pub fn delete(&mut self) {
        self.deleted = true;
    }

    pub fn restore(&mut self) {
        self.deleted = false;
    }

    pub fn enable_dragging(&mut self) {
        self.dragging = true;
    }

    pub fn disable_dragging(&mut self) {
        self.dragging = false;
    }

    pub fn is_dragging(&self) -> bool {
        self.dragging
    }

    pub fn set_position(&mut self, position: Option<Position>) {
        self.position = position;
    }

    pub fn position(&self) -> Option<Position> {
        self.position.clone()
    }

    pub fn stone(&self) -> Stone {
        self.stone.clone()
    }

    pub fn unique_key(&self) -> String {
        format!(
            "{}_{}",
            self.position
                .clone()
                .map(|p| p.to_string())
                .unwrap_or("deleted".to_string()),
            self.stone.char()
        )
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub struct ChessBoardSignals {
    chess_board: RwSignal<ChessBoard>,
    room_status: RwSignal<Option<RoomStatus>>,
    chess_board_socket: RwSignal<Option<WebSocket>, LocalStorage>,
    stones_signals: RwSignal<StonesSignals>,
    should_render: RwSignal<bool>,
    notification: RwSignal<Notification>,
    game: RwSignal<Option<GameInfo>>,
    /// `now_ms()` when the game's clock was received
    clock_synced_at: RwSignal<f64>,
    /// `now_ms()`, ticking while a clock runs
    clock_now: RwSignal<f64>,
    selected: RwSignal<Option<Selection>>,
    last_move: RwSignal<Option<(Option<Position>, Option<Position>)>>,
    pending_promotion: RwSignal<Option<PendingMove>>,
    connection: RwSignal<Connection>,
    /// The room in the URL, joined whenever the socket opens
    room_target: RwSignal<String>,
    needs_username: RwSignal<bool>,
    /// The game the user just created, the overlay navigates to it
    created_game: RwSignal<Option<String>>,
    /// A room to send the user to, e.g. when a game link is gone
    redirect: RwSignal<Option<String>>,
    /// The moves played so far, in algebraic notation
    moves: RwSignal<Vec<String>>,
}

#[allow(dead_code)]
impl ChessBoardSignals {
    pub fn new(chess_board: ChessBoard) -> Self {
        Self {
            chess_board: RwSignal::new(chess_board),
            room_status: RwSignal::new(None),
            // A `WebSocket` is a JS object, so its signal stays on the thread that made it
            chess_board_socket: RwSignal::new_local(None),
            stones_signals: RwSignal::new(StonesSignals::new()),
            should_render: RwSignal::new(false),
            notification: RwSignal::new(Notification::new(String::new(), NotifyType::Success)),
            game: RwSignal::new(None),
            clock_synced_at: RwSignal::new(0.0),
            clock_now: RwSignal::new(0.0),
            selected: RwSignal::new(None),
            last_move: RwSignal::new(None),
            pending_promotion: RwSignal::new(None),
            connection: RwSignal::new(Connection::Closed),
            room_target: RwSignal::new(String::new()),
            needs_username: RwSignal::new(false),
            created_game: RwSignal::new(None),
            redirect: RwSignal::new(None),
            moves: RwSignal::new(Vec::new()),
        }
    }

    pub fn socket(&self) -> RwSignal<Option<WebSocket>, LocalStorage> {
        self.chess_board_socket
    }

    pub fn room_status(&self) -> RwSignal<Option<RoomStatus>> {
        self.room_status
    }

    pub fn chess_board(&self) -> RwSignal<ChessBoard> {
        self.chess_board
    }

    pub fn stones_signals(&self) -> RwSignal<StonesSignals> {
        self.stones_signals
    }

    pub fn should_render(&self) -> RwSignal<bool> {
        self.should_render
    }

    pub fn notification(&self) -> RwSignal<Notification> {
        self.notification
    }

    pub fn game(&self) -> RwSignal<Option<GameInfo>> {
        self.game
    }

    pub fn clock_synced_at(&self) -> RwSignal<f64> {
        self.clock_synced_at
    }

    pub fn clock_now(&self) -> RwSignal<f64> {
        self.clock_now
    }

    pub fn selected(&self) -> RwSignal<Option<Selection>> {
        self.selected
    }

    pub fn last_move(&self) -> RwSignal<Option<(Option<Position>, Option<Position>)>> {
        self.last_move
    }

    pub fn pending_promotion(&self) -> RwSignal<Option<PendingMove>> {
        self.pending_promotion
    }

    pub fn connection(&self) -> RwSignal<Connection> {
        self.connection
    }

    pub fn room_target(&self) -> RwSignal<String> {
        self.room_target
    }

    pub fn needs_username(&self) -> RwSignal<bool> {
        self.needs_username
    }

    pub fn created_game(&self) -> RwSignal<Option<String>> {
        self.created_game
    }

    pub fn redirect(&self) -> RwSignal<Option<String>> {
        self.redirect
    }

    pub fn moves(&self) -> RwSignal<Vec<String>> {
        self.moves
    }

    /// Sends a message to the server, dropped while the socket is closed
    pub fn send(&self, msg: &ClientMsg) {
        let json = serde_json::to_string(msg).expect("client messages serialize");
        self.chess_board_socket.with_untracked(|socket| {
            if let Some(socket) = socket {
                if let Err(err) = socket.send_with_str(&json) {
                    log::error!("error sending message: {:?}", err);
                }
            }
        });
    }

    /// Takes the server's board after a move: turn, castling rights and en
    /// passant square come from it, local moves don't apply those effects.
    /// The pieces on screen are left alone, they already show the move.
    pub fn sync_position(&self, fen: &str, trash: &str) {
        let white_view = self.chess_board.with_untracked(|board| board.white_view());
        match ChessBoardBuilder::new()
            .fen(fen)
            .deleted_stones(trash)
            .is_white_view(white_view)
            .validation(false)
            .build()
        {
            Ok(board) => self.chess_board.set(board),
            Err(err) => log::error!("Invalid position {} {}: {:?}", fen, trash, err),
        }
    }

    pub fn notify(&self, level: NotifyType, message: &str) {
        self.notification.update(|notification| {
            notification.show(level, message);
        });
    }

    /// The user's seat in the room's game. Reads the game first so a view
    /// rendered before the session cookie exists still follows the game.
    pub fn my_seat(&self) -> Option<Side> {
        self.game.with(|game| game.as_ref()?.seat_of(&my_id()?))
    }

    pub fn is_game(&self) -> bool {
        self.game.with(Option::is_some)
    }

    /// Whether the rules are enforced on this board
    pub fn validation(&self) -> bool {
        self.room_status
            .with(|status| status.as_ref().is_some_and(|status| status.options().validation()))
    }

    /// Whether the user may pick up `piece` (`lp`, `dk`...). On a free board
    /// anyone moves anything; in a game only the seated player whose turn it
    /// is, and never from the trash.
    pub fn can_move(&self, piece: &str, from_trash: bool) -> bool {
        self.game.with_untracked(|game| {
            let Some(game) = game else {
                return true;
            };
            let Some(id) = my_id() else {
                return false;
            };
            let side = if piece.starts_with('l') {
                Side::White
            } else {
                Side::Black
            };

            !from_trash
                && game.result.is_none()
                && game.is_full()
                && game.seat_of(&id) == Some(side)
                && self.chess_board.with_untracked(|board| board.turn) == side
        })
    }

    /// A free board's checkmate, a game shows its result instead
    pub fn is_checkmate(&self) -> bool {
        !self.is_game()
            && self
                .room_status()
                .get()
                .map(|room_status| room_status.checkmate() && room_status.options().validation())
                .unwrap_or(false)
    }

    pub fn move_piece(&self, piece: String, old_pos: String, new_pos: String) {
        if old_pos == new_pos {
            return;
        }

        let (Ok(old_pos), Ok(new_pos)) = (
            Position::parse_square(&old_pos),
            Position::parse_square(&new_pos),
        ) else {
            log::error!("Invalid move squares: {} {}", old_pos, new_pos);
            return;
        };

        let old_pos_clone = old_pos.clone();
        let new_pos_clone = new_pos.clone();

        let stones_signals = self.stones_signals().get_untracked();
        let deleted_stones = stones_signals.deleted_stones();
        let mut new_pos_stone: Option<Stone> = None;
        if let Some(new_pos) = new_pos.clone() {
            let x = new_pos.x;
            let y = new_pos.y;
            new_pos_stone = self
                .chess_board()
                .get_untracked()
                .stone_at(x, y)
                .map(|s| s.clone());
        }

        self.chess_board().update(|chessboard| {
            let _ = chessboard.move_piece(&piece, old_pos_clone, new_pos_clone);
        });

        if old_pos.is_none() {
            let new_pos = new_pos.expect("new_pos should be Some");

            let stone_key = deleted_stones
                .iter()
                .find(|(_, stone)| *stone.get_untracked().stone().image_class() == piece);

            if let Some(new_pos_stone) = new_pos_stone {
                let stone_key =
                    StoneSignal::new(Some(new_pos.clone()), new_pos_stone.clone()).unique_key();

                if let Some(stone_signal) = self
                    .stones_signals()
                    .with_untracked(|ss| ss.get_board_stone(&stone_key))
                {
                    stone_signal.update(|ss| {
                        ss.set_position(None);
                        ss.delete();
                    });
                };

                self.stones_signals().update(|stones| {
                    if let Some(stone) = stones.remove_board_stone(stone_key) {
                        stones.add_deleted_stone_signal(stone);
                    };
                });
            }

            let new_key = StoneSignal::new(
                Some(new_pos.clone()),
                piece
                    .parse::<Stone>()
                    .expect("piece_data should be a valid stone"),
            )
            .unique_key();

            if let Some((key, stone_signal)) = stone_key.map(|sk| sk) {
                stone_signal.update(|ss| {
                    ss.set_position(Some(new_pos.clone()));
                    ss.disable_dragging();
                    ss.restore();
                });

                let key = key.clone();
                self.stones_signals().update(|stones| {
                    if let Some(stone) = stones.remove_deleted_stone(key) {
                        stones.add_board_stone_signal(new_key, stone);
                    };
                });
            }
        } else if new_pos.is_none() {
            let old_pos = old_pos.expect("old_pos should be Some");

            let stone_signal_key = StoneSignal::new(
                Some(old_pos),
                piece
                    .parse::<Stone>()
                    .expect("piece_data should be a valid stone"),
            )
            .unique_key();

            let mut stone_signal: Option<RwSignal<StoneSignal>> = None;

            self.stones_signals().update(|stones| {
                if let Some(stone) = stones.remove_board_stone(stone_signal_key) {
                    stones.add_deleted_stone_signal(stone);
                    stone_signal = Some(stone);
                }
            });

            if let Some(stone_signal) = stone_signal {
                stone_signal.update(|ss| {
                    ss.set_position(None);
                    ss.delete();
                    ss.disable_dragging();
                });
            }
        } else {
            let old_pos = old_pos.expect("old_pos should be Some");
            let new_pos = new_pos.expect("new_pos should be Some");

            let stone_signal_key = StoneSignal::new(
                Some(old_pos),
                piece
                    .parse::<Stone>()
                    .expect("piece_data should be a valid stone"),
            )
            .unique_key();

            let new_pos_stone_signal_key = new_pos_stone
                .map(|stone| StoneSignal::new(Some(new_pos.clone()), stone.clone()).unique_key());

            let mut stone_signal: Option<RwSignal<StoneSignal>> = None;
            let mut deleted_stone_signal: Option<RwSignal<StoneSignal>> = None;
            let new_key = StoneSignal::new(
                Some(new_pos.clone()),
                piece
                    .parse::<Stone>()
                    .expect("piece_data should be a valid stone"),
            )
            .unique_key();

            if let Some(key) = new_pos_stone_signal_key.clone() {
                if let Some(stone_signal) = self
                    .stones_signals()
                    .with_untracked(|ss| ss.get_board_stone(&key))
                {
                    stone_signal.update(|ss| {
                        ss.set_position(None);
                        ss.delete();
                    });
                };

                self.stones_signals.update(|stones| {
                    if let Some(stone) = stones.remove_board_stone(key) {
                        stones.add_deleted_stone_signal(stone);
                        deleted_stone_signal = Some(stone);
                    };
                });
            }

            self.stones_signals().update(|stones| {
                if let Some(stone) = stones.remove_board_stone(stone_signal_key) {
                    stones.add_board_stone_signal(new_key, stone);
                    stone_signal = Some(stone);
                }
            });

            if let Some(deleted_stone_signal) = deleted_stone_signal {
                deleted_stone_signal.update(|ss| {
                    ss.set_position(None);
                    ss.disable_dragging();
                });
            }

            if let Some(stone_signal) = stone_signal {
                stone_signal.update(|ss| {
                    ss.set_position(Some(new_pos.clone()));
                    ss.disable_dragging();
                });
            }
        }
    }

    #[allow(unused_variables)]
    pub fn start_websocket(&self) {
        if self.chess_board_socket.with_untracked(|ws| ws.is_some()) {
            return;
        }

        cfg_if! {
            if #[cfg(not(feature = "ssr"))] {
                let ws = crate::client::websockets::chess_board::start_websocket(*self).ok();
            } else {
                let ws = None;
            }
        }

        self.chess_board_socket.set(ws);
    }
}
