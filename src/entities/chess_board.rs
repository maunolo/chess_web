use std::collections::HashSet;
use std::ops::Not;

use cfg_if::cfg_if;
use leptos::{RwSignal, Scope, SignalSet, SignalWithUntracked, WriteSignal};
use web_sys::WebSocket;

use super::position::Position;
use super::room::RoomStatus;
use super::stone::{Color, Kind, Stone};

pub fn fen_to_passant(field: &str) -> Result<Option<Position>, ()> {
    if field == "-" {
        return Ok(None);
    }

    let x = field.chars().nth(0).ok_or(())? as usize - 97;
    let y = 8 - field.chars().nth(1).ok_or(())?.to_digit(10).ok_or(())? as usize;
    Ok(Some(Position::new(x, y)))
}

pub fn fen_to_castle_rules(field: &str) -> Result<CastleRules, ()> {
    let mut white_castle_options = CastleOptions::None;
    let mut black_castle_options = CastleOptions::None;

    for c in field.chars() {
        match c {
            'K' => {
                white_castle_options = CastleOptions::KingSide;
            }
            'Q' => match white_castle_options {
                CastleOptions::KingSide => {
                    white_castle_options = CastleOptions::BothSides;
                }
                _ => {
                    white_castle_options = CastleOptions::QueenSide;
                }
            },
            'k' => {
                black_castle_options = CastleOptions::KingSide;
            }
            'q' => match black_castle_options {
                CastleOptions::KingSide => {
                    black_castle_options = CastleOptions::BothSides;
                }
                _ => {
                    black_castle_options = CastleOptions::QueenSide;
                }
            },
            '-' => {
                break;
            }
            _ => {
                return Err(());
            }
        }
    }

    Ok(CastleRules {
        white: white_castle_options,
        black: black_castle_options,
    })
}

pub fn fen_to_stones(field: &str) -> Result<[[Option<Stone>; 8]; 8], ()> {
    const INIT: Option<Stone> = None;
    const ROW: [Option<Stone>; 8] = [INIT; 8];
    let mut stones = [ROW; 8];

    for (y, row) in field.split("/").enumerate() {
        if y > 7 {
            return Err(());
        }

        let mut empty_squares = 0;

        for (x, c) in row.chars().enumerate() {
            if x > 7 {
                return Err(());
            }

            if c.is_digit(10) {
                let n = c.to_digit(10).ok_or(())?;
                let x = x + empty_squares as usize;
                empty_squares += n - 1;
                for i in 0..n {
                    let x = x + i as usize;
                    if x > 7 {
                        return Err(());
                    }
                    stones[y][x] = None;
                }
            } else {
                let x = x + empty_squares as usize;
                if x > 7 {
                    return Err(());
                }
                stones[y][x] = Some(Stone::try_from(c)?);
            }
        }
    }

    Ok(stones)
}

pub fn fen_to_turn(field: &str) -> Result<Turn, ()> {
    if field == "w" {
        Ok(Turn::White)
    } else if field == "b" {
        Ok(Turn::Black)
    } else {
        Err(())
    }
}

#[derive(Clone)]
pub enum CastleOptions {
    KingSide,
    QueenSide,
    BothSides,
    None,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct CastleRules {
    white: CastleOptions,
    black: CastleOptions,
}

impl CastleRules {
    pub fn white(&self) -> &CastleOptions {
        &self.white
    }

    pub fn black(&self) -> &CastleOptions {
        &self.black
    }
}

#[derive(Clone, Copy)]
pub enum Turn {
    White,
    Black,
}

impl Not for Turn {
    type Output = Turn;

    fn not(self) -> Self::Output {
        match self {
            Turn::White => Turn::Black,
            Turn::Black => Turn::White,
        }
    }
}

#[derive(Clone)]
pub struct ChessBoard {
    pub fen: String,
    pub stones: [[Option<Stone>; 8]; 8],
    pub treat_map: [[bool; 8]; 8],
    pub turn: Turn,
    pub castle_rules: CastleRules,
    pub passant: Option<Position>,
    pub half_move_clock: i32,
    pub full_move_clock: i32,
    pub deleted_stones: Vec<Stone>,
    pub is_white_view: bool,
    pub validation: bool,
}

#[allow(dead_code)]
impl ChessBoard {
    pub fn new(fen: &str) -> Result<Self, ()> {
        let fen_fields = fen.split(" ").collect::<Vec<&str>>();
        let mut chess_board = Self {
            fen: fen.to_string(),
            stones: fen_to_stones(fen_fields[0])?,
            treat_map: [[false; 8]; 8],
            turn: fen_to_turn(fen_fields[1])?,
            castle_rules: fen_to_castle_rules(fen_fields[2])?,
            passant: fen_to_passant(fen_fields[3])?,
            half_move_clock: fen_fields[4].parse::<i32>().map_err(|_| ())?,
            full_move_clock: fen_fields[5].parse::<i32>().map_err(|_| ())?,
            deleted_stones: Vec::new(),
            is_white_view: true,
            validation: false,
        };
        chess_board.sync_treat_map();
        if !chess_board.valid_castle_rules() {
            return Err(());
        }
        Ok(chess_board)
    }

    pub fn is_in_check(&self) -> bool {
        let Some((position, _)) = self.stones_and_positions_iter().find(|(_, stone)| {
            matches!(stone.kind(), Kind::King)
                && match self.turn {
                    Turn::White => matches!(stone.color(), Color::Light),
                    Turn::Black => matches!(stone.color(), Color::Dark),
                }
        }) else {
            return false;
        };

        self.treat_at(position.x, position.y)
    }

    pub fn valid_castle_rules(&self) -> bool {
        match self.castle_rules.white() {
            CastleOptions::KingSide => {
                if !self.stone_at_is(5, 7, Kind::King) {
                    return false;
                }

                if !self.stone_at_is(7, 7, Kind::Rook) {
                    return false;
                }
            }
            CastleOptions::QueenSide => {
                if !self.stone_at_is(5, 7, Kind::King) {
                    return false;
                }

                if !self.stone_at_is(0, 7, Kind::Rook) {
                    return false;
                }
            }
            CastleOptions::BothSides => {
                if !self.stone_at_is(5, 7, Kind::King) {
                    return false;
                }

                if !self.stone_at_is(7, 7, Kind::Rook) {
                    return false;
                }

                if !self.stone_at_is(0, 7, Kind::Rook) {
                    return false;
                }
            }
            _ => {}
        }

        match self.castle_rules.black() {
            CastleOptions::KingSide => {
                if !self.stone_at_is(5, 0, Kind::King) {
                    return false;
                }

                if !self.stone_at_is(7, 0, Kind::Rook) {
                    return false;
                }
            }
            CastleOptions::QueenSide => {
                if !self.stone_at_is(5, 0, Kind::King) {
                    return false;
                }

                if !self.stone_at_is(0, 0, Kind::Rook) {
                    return false;
                }
            }
            CastleOptions::BothSides => {
                if !self.stone_at_is(5, 0, Kind::King) {
                    return false;
                }

                if !self.stone_at_is(7, 0, Kind::Rook) {
                    return false;
                }

                if !self.stone_at_is(0, 0, Kind::Rook) {
                    return false;
                }
            }
            _ => {}
        }

        true
    }

    #[allow(unused_variables)]
    pub fn stone_at_is(&self, x: usize, y: usize, kind: Kind) -> bool {
        self.stone_at(x, y)
            .map(|s| matches!(s.kind(), kind))
            .unwrap_or(false)
    }

    pub fn toggle_validation(&mut self) {
        self.validation = !self.validation;
    }

    pub fn castle_rules(&self) -> &CastleRules {
        &self.castle_rules
    }

    pub fn stone_at(&self, x: usize, y: usize) -> Option<&Stone> {
        self.stones[y][x].as_ref()
    }

    pub fn take_stone_at(&mut self, x: usize, y: usize) -> Option<Stone> {
        self.stones[y][x].take()
    }

    pub fn cloned_stones_and_positions(&self) -> Vec<(Position, Stone)> {
        self.stones_and_positions_iter()
            .map(|(position, stone)| (position, stone.clone()))
            .collect()
    }

    pub fn stones_and_positions(&self) -> Vec<(Position, &Stone)> {
        self.stones_and_positions_iter().collect()
    }

    pub fn stones_and_positions_iter(&self) -> impl Iterator<Item = (Position, &Stone)> + '_ {
        self.stones
            .iter()
            .enumerate()
            .flat_map(|(y, row)| {
                let row_pieces: Vec<(Position, &Stone)> = row
                    .iter()
                    .enumerate()
                    .filter_map(|(x, stone)| {
                        let stone = stone.as_ref()?;
                        let position = Position::new(x, y);
                        Some((position, stone))
                    })
                    .collect();
                row_pieces
            })
            .into_iter()
    }

    pub fn deleted_stones(&self) -> Vec<Stone> {
        self.deleted_stones.clone()
    }

    pub fn flip(&mut self) {
        self.is_white_view = !self.is_white_view;
    }

    pub fn trash_string(&self) -> String {
        self.deleted_stones.iter().map(|s| s.char()).collect()
    }

    pub fn set_trash_from_str(&mut self, trash: &str) {
        for c in trash.chars() {
            let Ok(stone) = Stone::try_from(c) else {
                log::error!("Error parsing stone from char: {:?}", c);
                continue;
            };
            self.deleted_stones.push(stone);
        }
    }

    pub fn set_treat(&mut self, x: usize, y: usize) {
        self.treat_map[y][x] = true;
    }

    pub fn treat_at(&self, x: usize, y: usize) -> bool {
        self.treat_map[y][x]
    }

    pub fn free_at(&self, x: usize, y: usize) -> bool {
        self.stone_at(x, y).is_none() && !self.treat_at(x, y)
    }

    pub fn set_treat_map(&mut self, turn: Turn) {
        self.treat_map = [[false; 8]; 8];

        let mut treat_moves = HashSet::new();

        match turn {
            Turn::White => {
                for (position, stone) in self
                    .stones_and_positions_iter()
                    .filter(|(_, stone)| matches!(stone.color(), Color::Dark))
                {
                    for move_pos in stone.possible_moves(&position, &self) {
                        treat_moves.insert(move_pos);
                    }
                }
            }
            Turn::Black => {
                for (position, stone) in self
                    .stones_and_positions_iter()
                    .filter(|(_, stone)| matches!(stone.color(), Color::Light))
                {
                    for move_pos in stone.possible_moves(&position, &self) {
                        treat_moves.insert(move_pos);
                    }
                }
            }
        }

        for position in treat_moves {
            self.set_treat(position.x, position.y);
        }
    }

    pub fn sync_treat_map(&mut self) {
        self.set_treat_map(self.turn);
    }

    pub fn possible_moves(&self, position: &Position) -> Vec<Position> {
        let Some(ref stone) = self.stone_at(position.x, position.y) else {
            return Vec::new();
        };
        let mut moves = Vec::new();
        for possible_move in stone.possible_moves(position, &self) {
            let mut chess_board = self.clone();
            chess_board.move_piece(
                &stone.image_class(),
                Some(position.clone()),
                Some(possible_move.clone()),
            );
            chess_board.turn = !chess_board.turn;
            chess_board.sync_treat_map();
            if !chess_board.is_in_check() {
                moves.push(possible_move);
            }
        }
        moves
    }

    pub fn move_piece(&mut self, piece: &str, from: Option<Position>, to: Option<Position>) {
        if from == to {
            return;
        }

        if from.is_none() {
            if let Some(stone_idx) = self
                .deleted_stones
                .iter()
                .position(|s| s.image_class() == piece)
            {
                let stone = self.deleted_stones.remove(stone_idx);
                let to = to.unwrap();
                let old_piece = self.take_stone_at(to.x, to.y);
                self.stones[to.y as usize][to.x as usize] = Some(stone);
                if let Some(old_piece) = old_piece {
                    self.deleted_stones.push(old_piece);
                }
            };
        } else if to.is_none() {
            let from = from.unwrap();
            let stone = self.take_stone_at(from.x, from.y);
            if let Some(stone) = stone {
                self.deleted_stones.push(stone);
            }
        } else {
            let from = from.unwrap();
            let to = to.unwrap();
            let old_piece = self.take_stone_at(to.x, to.y);
            let stone = self.take_stone_at(from.x, from.y);
            self.stones[to.y as usize][to.x as usize] = stone;
            if let Some(old_piece) = old_piece {
                self.deleted_stones.push(old_piece);
            }
        }

        self.turn = !self.turn;
        self.sync_fen();
        self.sync_treat_map();
    }

    pub fn sync_fen(&mut self) {
        let mut new_fen = String::new();
        for (i, row) in self.stones.iter().enumerate() {
            let mut empty = 0;
            for stone in row {
                if let Some(stone) = stone {
                    if empty > 0 {
                        new_fen.push_str(&empty.to_string());
                        empty = 0;
                    }
                    new_fen.push_str(&stone.char().to_string());
                } else {
                    empty += 1;
                }
            }
            if empty > 0 {
                new_fen.push_str(&empty.to_string());
            }
            if i < 7 {
                new_fen.push('/');
            }
        }
        if matches!(self.turn, Turn::White) {
            new_fen.push_str(" w");
        } else {
            new_fen.push_str(" b");
        }
        let fen_clone = self.fen.clone();
        let fen_split = fen_clone.split(" ").skip(2);
        let fen_rest = fen_split.collect::<Vec<&str>>().join(" ");
        new_fen.push_str(&format!(" {}", fen_rest));

        self.fen = new_fen;
    }

    pub fn css_class(&self) -> String {
        if self.is_white_view {
            "chessboard"
        } else {
            "chessboard flipped"
        }
        .to_string()
    }

    pub fn white_view(&self) -> bool {
        self.is_white_view.clone()
    }
}

pub struct ChessBoardSignalsBuilder {
    cx: Option<Scope>,
    chess_board: Option<WriteSignal<ChessBoard>>,
    should_render: Option<WriteSignal<bool>>,
    room_status: Option<RwSignal<Option<RoomStatus>>>,
    chess_board_socket: Option<RwSignal<Option<WebSocket>>>,
}

impl ChessBoardSignalsBuilder {
    pub fn new() -> Self {
        Self {
            cx: None,
            chess_board: None,
            should_render: None,
            room_status: None,
            chess_board_socket: None,
        }
    }

    pub fn cx(mut self, cx: Scope) -> Self {
        self.cx = Some(cx);
        self
    }

    pub fn chess_board(mut self, chess_board: WriteSignal<ChessBoard>) -> Self {
        self.chess_board = Some(chess_board);
        self
    }

    pub fn should_render(mut self, should_render: WriteSignal<bool>) -> Self {
        self.should_render = Some(should_render);
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

    pub fn build(self) -> Result<ChessBoardSignals, ()> {
        let Some(cx) = self.cx else {
            return Err(());
        };
        let Some(chess_board) = self.chess_board else {
            return Err(());
        };
        let Some(should_render) = self.should_render else {
            return Err(());
        };
        let Some(room_status) = self.room_status else {
            return Err(());
        };
        let Some(chess_board_socket) = self.chess_board_socket else {
            return Err(());
        };

        Ok(ChessBoardSignals {
            cx,
            chess_board,
            should_render,
            room_status,
            chess_board_socket,
        })
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub struct ChessBoardSignals {
    cx: Scope,
    chess_board: WriteSignal<ChessBoard>,
    should_render: WriteSignal<bool>,
    room_status: RwSignal<Option<RoomStatus>>,
    chess_board_socket: RwSignal<Option<WebSocket>>,
}

#[allow(dead_code)]
impl ChessBoardSignals {
    pub fn cx(&self) -> Scope {
        self.cx.clone()
    }

    pub fn socket(&self) -> RwSignal<Option<WebSocket>> {
        self.chess_board_socket
    }

    pub fn room_status(&self) -> RwSignal<Option<RoomStatus>> {
        self.room_status
    }

    pub fn chess_board(&self) -> WriteSignal<ChessBoard> {
        self.chess_board
    }

    pub fn should_render(&self) -> WriteSignal<bool> {
        self.should_render
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chess_board() {
        let fen = "4kbnr/p3pppp/3b4/8/8/8/PPP2PPP/RNBQKBNR b KQk - 0 1";
        let chess_board = ChessBoard::new(fen).unwrap();

        assert_eq!(
            Vec::<Position>::new(),
            chess_board.possible_moves(&Position::new(3, 2))
        );
    }
}
