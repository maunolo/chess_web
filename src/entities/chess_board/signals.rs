use cfg_if::cfg_if;
use leptos::{
    create_rw_signal, RwSignal, SignalGet, SignalGetUntracked, SignalSet, SignalUpdate,
    SignalWithUntracked,
};
use std::collections::BTreeMap;
use web_sys::WebSocket;

use super::ChessBoard;

use crate::entities::{
    notification::Notification, position::Position, room::RoomStatus, stone::Stone,
};

pub struct ChessBoardSignalsBuilder {
    chess_board: Option<RwSignal<ChessBoard>>,
    room_status: Option<RwSignal<Option<RoomStatus>>>,
    chess_board_socket: Option<RwSignal<Option<WebSocket>>>,
    stones_signals: Option<RwSignal<StonesSignals>>,
    should_render: Option<RwSignal<bool>>,
    notification: Option<RwSignal<Notification>>,
}

impl ChessBoardSignalsBuilder {
    pub fn new() -> Self {
        Self {
            chess_board: None,
            room_status: None,
            chess_board_socket: None,
            stones_signals: None,
            should_render: None,
            notification: None,
        }
    }

    pub fn chess_board(mut self, chess_board: RwSignal<ChessBoard>) -> Self {
        self.chess_board = Some(chess_board);
        self
    }

    pub fn room_status(mut self, room_status: RwSignal<Option<RoomStatus>>) -> Self {
        self.room_status = Some(room_status);
        self
    }

    pub fn chess_board_socket(mut self, chess_board_socket: RwSignal<Option<WebSocket>>) -> Self {
        self.chess_board_socket = Some(chess_board_socket);
        self
    }

    pub fn stones_signals(mut self, stones_signals: RwSignal<StonesSignals>) -> Self {
        self.stones_signals = Some(stones_signals);
        self
    }

    pub fn should_render(mut self, should_render: RwSignal<bool>) -> Self {
        self.should_render = Some(should_render);
        self
    }

    pub fn notification(mut self, notification: RwSignal<Notification>) -> Self {
        self.notification = Some(notification);
        self
    }

    pub fn build(self) -> Result<ChessBoardSignals, ()> {
        let Some(chess_board) = self.chess_board else {
            return Err(());
        };
        let Some(room_status) = self.room_status else {
            return Err(());
        };
        let Some(chess_board_socket) = self.chess_board_socket else {
            return Err(());
        };
        let Some(stones_signals) = self.stones_signals else {
            return Err(());
        };
        let Some(should_render) = self.should_render else {
            return Err(());
        };
        let Some(notification) = self.notification else {
            return Err(());
        };

        Ok(ChessBoardSignals {
            chess_board,
            room_status,
            chess_board_socket,
            stones_signals,
            should_render,
            notification,
        })
    }
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
        let stone_signal = create_rw_signal(stone_signal);

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
        let stone_signal = create_rw_signal(stone_signal);
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
    chess_board_socket: RwSignal<Option<WebSocket>>,
    stones_signals: RwSignal<StonesSignals>,
    should_render: RwSignal<bool>,
    notification: RwSignal<Notification>,
}

#[allow(dead_code)]
impl ChessBoardSignals {
    pub fn socket(&self) -> RwSignal<Option<WebSocket>> {
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

    pub fn is_checkmate(&self) -> bool {
        self.room_status()
            .get()
            .map(|room_status| room_status.checkmate() && room_status.options().validation())
            .unwrap_or(false)
    }

    pub fn move_piece(&self, piece: String, old_pos: String, new_pos: String) {
        if old_pos == new_pos {
            return;
        }

        let old_pos = match old_pos.as_str() {
            "deleted" => None,
            _ => Some(old_pos.parse::<Position>().unwrap()),
        };
        let new_pos = match new_pos.as_str() {
            "deleted" => None,
            _ => Some(new_pos.parse::<Position>().unwrap()),
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
