use std::collections::HashSet;

use crate::entities::chess_board::enums::{PromotionKind, CastlePosition};

use self::castle_rules::{fen_to_castle_rules, CastleRules, CastleOptions};
use self::enums::{FenError, ChessBoardError, Move, MoveError};
use self::passants::fen_to_passant;
use self::stones::fen_to_stones;
use self::turns::{fen_to_turn, Turn};

use super::position::Position;
use super::stone::{Color, Kind, Stone};

pub mod castle_rules;
pub mod enums;
pub mod stones;
pub mod turns;
pub mod passants;
pub mod signals;

pub struct ChessBoardBuilder {
    fen: Option<String>,
    deleted_stones: Option<String>,
    is_white_view: Option<bool>,
    validation: Option<bool>,
    sync: Option<bool>,
}

#[allow(dead_code)]
impl ChessBoardBuilder {
    pub fn new() -> Self {
        Self {
            fen: None,
            deleted_stones: Some(String::new()),
            is_white_view: Some(true),
            validation: Some(true),
            sync: Some(true),
        }
    }

    pub fn fen(mut self, fen: &str) -> Self {
        self.fen = Some(fen.to_owned());
        self
    }

    pub fn deleted_stones(mut self, deleted_stones: &str) -> Self {
        self.deleted_stones = Some(deleted_stones.to_owned());
        self
    }

    pub fn is_white_view(mut self, is_white_view: bool) -> Self {
        self.is_white_view = Some(is_white_view);
        self
    }

    pub fn validation(mut self, validation: bool) -> Self {
        self.validation = Some(validation);
        self
    }

    pub fn sync(mut self, sync: bool) -> Self {
        self.sync = Some(sync);
        self
    }

    pub fn build(self) -> Result<ChessBoard, ChessBoardError> {
        let Some(fen) = self.fen else {
            return Err(ChessBoardError::BuildError);
        };

        let Some(deleted_stones_str) = self.deleted_stones else {
            return Err(ChessBoardError::BuildError);
        };

        let Some(is_white_view) = self.is_white_view else {
            return Err(ChessBoardError::BuildError);
        };

        let Some(validation) = self.validation else {
            return Err(ChessBoardError::BuildError);
        };

        let Some(sync) = self.sync else {
            return Err(ChessBoardError::BuildError);
        };

        let fen_fields = fen.split(" ").collect::<Vec<&str>>();
        if fen_fields.len() != 6 {
            return Err(ChessBoardError::InvalidFen(FenError::InvalidFormat));
        }
        let mut deleted_stones = Vec::new();
        for c in deleted_stones_str.chars() {
            deleted_stones.push(Stone::try_from(c).map_err(|_| ChessBoardError::InvalidDeletedStones)?);
        }

        let mut chess_board = ChessBoard {
            fen: fen.to_string(),
            stones: fen_to_stones(fen_fields[0]).map_err(|e| ChessBoardError::InvalidFen(e))?,
            treat_map: [[false; 8]; 8],
            turn: fen_to_turn(fen_fields[1]).map_err(|e| ChessBoardError::InvalidFen(e))?,
            castle_rules: fen_to_castle_rules(fen_fields[2]).map_err(|e| ChessBoardError::InvalidFen(e))?,
            passant: fen_to_passant(fen_fields[3]).map_err(|e| ChessBoardError::InvalidFen(e))?,
            half_move_clock: fen_fields[4].parse::<i32>().map_err(|_| ChessBoardError::InvalidFen(FenError::InvalidHalfMoveClock))?,
            full_move_clock: fen_fields[5].parse::<i32>().map_err(|_| ChessBoardError::InvalidFen(FenError::InvalidFullMoveClock))?,
            deleted_stones,
            is_white_view,
            validation,
            sync,
        };

        if chess_board.sync {
            chess_board.sync_treat_map();
        }

        if chess_board.validation && !chess_board.valid_castle_rules() {
            return Err(ChessBoardError::InvalidFen(FenError::InvalidCastleRules));
        }

        Ok(chess_board)
    }
}

#[derive(Clone, Debug)]
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
    pub sync: bool,
}

#[allow(dead_code)]
impl ChessBoard {
    pub fn new(fen: &str) -> Result<Self, ChessBoardError> {
        let fen_fields = fen.split(" ").collect::<Vec<&str>>();
        let mut chess_board = Self {
            fen: fen.to_string(),
            stones: fen_to_stones(fen_fields[0]).map_err(|e| ChessBoardError::InvalidFen(e))?,
            treat_map: [[false; 8]; 8],
            turn: fen_to_turn(fen_fields[1]).map_err(|e| ChessBoardError::InvalidFen(e))?,
            castle_rules: fen_to_castle_rules(fen_fields[2]).map_err(|e| ChessBoardError::InvalidFen(e))?,
            passant: fen_to_passant(fen_fields[3]).map_err(|e| ChessBoardError::InvalidFen(e))?,
            half_move_clock: fen_fields[4].parse::<i32>().map_err(|_| ChessBoardError::InvalidFen(FenError::InvalidHalfMoveClock))?,
            full_move_clock: fen_fields[5].parse::<i32>().map_err(|_| ChessBoardError::InvalidFen(FenError::InvalidFullMoveClock))?,
            deleted_stones: Vec::new(),
            is_white_view: true,
            validation: false,
            sync: true,
        };
        if chess_board.sync {
            chess_board.sync_treat_map();
        }
        if chess_board.validation && !chess_board.valid_castle_rules() {
            return Err(ChessBoardError::InvalidFen(FenError::InvalidCastleRules));
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
                if !self.stone_at_is(4, 7, Kind::King) {
                    return false;
                }

                if !self.stone_at_is(7, 7, Kind::Rook) {
                    return false;
                }
            }
            CastleOptions::QueenSide => {
                if !self.stone_at_is(4, 7, Kind::King) {
                    return false;
                }

                if !self.stone_at_is(0, 7, Kind::Rook) {
                    return false;
                }
            }
            CastleOptions::BothSides => {
                if !self.stone_at_is(4, 7, Kind::King) {
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
                if !self.stone_at_is(4, 0, Kind::King) {
                    return false;
                }

                if !self.stone_at_is(7, 0, Kind::Rook) {
                    return false;
                }
            }
            CastleOptions::QueenSide => {
                if !self.stone_at_is(4, 0, Kind::King) {
                    return false;
                }

                if !self.stone_at_is(0, 0, Kind::Rook) {
                    return false;
                }
            }
            CastleOptions::BothSides => {
                if !self.stone_at_is(4, 0, Kind::King) {
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

    pub fn cloned_deleted_stones(&self) -> Vec<Stone> {
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

    pub fn sync_treat_map(&mut self) {
        self.treat_map = [[false; 8]; 8];

        let mut treat_moves = HashSet::new();

        match self.turn {
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

    pub fn possible_moves(&self, position: &Position) -> HashSet<Position> {
        let Some(ref stone) = self.stone_at(position.x, position.y) else {
            return HashSet::new();
        };
        let mut moves = HashSet::new();
        for possible_move in stone.possible_moves(position, &self) {
            let mut chess_board = self.clone();
            chess_board.validation = false;
            chess_board.sync = false;
            let _ = chess_board.move_piece(
                &stone.image_class(),
                Some(position.clone()),
                Some(possible_move.clone()),
            );
            chess_board.sync_treat_map();
            if !chess_board.is_in_check() {
                moves.insert(possible_move);
            }
        }
        moves
    }

    pub fn move_piece(
        &mut self,
        piece: &str,
        from: Option<Position>,
        to: Option<Position>,
    ) -> Result<Move, ChessBoardError> {
        if from == to {
            return Ok(Move::Normal);
        }

        if self.validation {
            let stone = piece.parse::<Stone>().map_err(|_| ChessBoardError::InvalidMove(MoveError::NoStoneFound))?;

            match (stone.color(), self.turn) {
                (Color::Light, Turn::White) | (Color::Dark, Turn::Black) => {}
                _ => return Err(ChessBoardError::InvalidMove(MoveError::InvalidMove)),
            }

            if from.is_none() || to.is_none() {
                return Err(ChessBoardError::InvalidMove(MoveError::InvalidMove));
            }

            let to = to.clone().unwrap();
            let from = from.clone().unwrap();
            let possible_moves = self.possible_moves(&from);
            if !possible_moves.contains(&to) {
                return Err(ChessBoardError::InvalidMove(MoveError::InvalidMove));
            }
        }

        let Some(stone) = (match from.clone() {
            None => {
                if let Some(idx) = self
                    .deleted_stones
                    .iter()
                    .position(|s| s.image_class() == piece)
                    {
                        Some(self.deleted_stones.remove(idx))
                    } else {
                        return Err(ChessBoardError::InvalidMove(MoveError::NoStoneFound));
                    }
            }
            Some(from) => self.take_stone_at(from.x, from.y) 
        }) else {
            return Err(ChessBoardError::InvalidMove(MoveError::NoStoneFound));
        };

        let result;

        if from.is_none() {
            let to = to.unwrap();
            let old_piece = self.take_stone_at(to.x, to.y);
            self.stones[to.y as usize][to.x as usize] = Some(stone);
            if let Some(old_piece) = old_piece {
                self.deleted_stones.push(old_piece);
            }
            result = Move::Normal;
        } else if to.is_none() {
            self.deleted_stones.push(stone);
            result = Move::Normal;
        } else {
            let from = from.unwrap();
            let to = to.unwrap(); 

            let old_piece = self.take_stone_at(to.x, to.y);
            self.stones[to.y as usize][to.x as usize] = Some(stone.clone());
            if let Some(old_piece) = old_piece {
                self.deleted_stones.push(old_piece);
            } 

            result = if self.validation {
                self.apply_move_validation_and_effects((&stone, &from, &to))
            } else {
                Move::Normal
            }
        }

        if self.sync {
            self.turn = !self.turn;
            self.sync_fen();
            self.sync_treat_map();
        }

        Ok(result)
    }

    pub fn apply_move_validation_and_effects(&mut self, stone_move: (&Stone, &Position, &Position)) -> Move {
        match stone_move {
            (stone, _, to) if Some(to.clone()) == self.passant => {
                let passant_pos = to.clone();
                let passant_stone = match stone.color() {
                    Color::Light => {
                        self.take_stone_at(passant_pos.x, passant_pos.y + 1)
                    }
                    Color::Dark => {
                        self.take_stone_at(passant_pos.x, passant_pos.y - 1)
                    }
                };
                self.deleted_stones.push(passant_stone.unwrap());
                self.passant = None;
                Move::Passant
            }
            (stone, from, to) if stone.as_str() == "lp" && from.y == 6 && to.y == 4 => {
                self.passant = Some(Position::new(to.x, 5));
                Move::Normal
            }
            (stone, from, to) if stone.as_str() == "dp" && from.y == 1 && to.y == 3 => {
                self.passant = Some(Position::new(to.x, 2));
                Move::Normal
            }
            (stone, _, to) if stone.as_str() == "lp" && to.y == 0 => {
                self.passant = None;
                let new_stone: Stone = match PromotionKind::Queen {
                    PromotionKind::Queen => 'Q'.try_into().unwrap(),
                    PromotionKind::Rook => 'R'.try_into().unwrap(),
                    PromotionKind::Bishop => 'B'.try_into().unwrap(),
                    PromotionKind::Knight => 'N'.try_into().unwrap(),
                };
                self.stones[to.y as usize][to.x as usize] = Some(new_stone);
                Move::Promotion(PromotionKind::Queen)
            }
            (stone, _, to) if stone.as_str() == "dp" && to.y == 7 => {
                self.passant = None;
                let new_stone: Stone = match PromotionKind::Queen {
                    PromotionKind::Queen => 'q'.try_into().unwrap(),
                    PromotionKind::Rook => 'r'.try_into().unwrap(),
                    PromotionKind::Bishop => 'b'.try_into().unwrap(),
                    PromotionKind::Knight => 'n'.try_into().unwrap(),
                };
                self.stones[to.y as usize][to.x as usize] = Some(new_stone);
                Move::Promotion(PromotionKind::Queen)
            }
            (stone, from, to) if stone.as_str() == "lk" && from.x == 4 && from.y == 7 && to.x == 6 && to.y == 7 => {
                let rook = self.take_stone_at(7, 7);
                self.stones[7][5] = rook;
                self.passant = None;
                self.castle_rules.white = CastleOptions::None;
                Move::Castle(CastlePosition::KingSide)
            }
            (stone, from, to) if stone.as_str() == "lk" && from.x == 4 && from.y == 7 && to.x == 2 && to.y == 7 => {
                let rook = self.take_stone_at(0, 7);
                self.stones[7][3] = rook;
                self.passant = None;
                self.castle_rules.white = CastleOptions::None;
                Move::Castle(CastlePosition::QueenSide)
            }
            (stone, from, to) if stone.as_str() == "dk" && from.x == 4 && from.y == 0 && to.x == 6 && to.y == 0 => {
                let rook = self.take_stone_at(7, 0);
                self.stones[0][5] = rook;
                self.passant = None;
                self.castle_rules.black = CastleOptions::None;
                Move::Castle(CastlePosition::KingSide)              
            }
            (stone, from, to) if stone.as_str() == "dk" && from.x == 4 && from.y == 0 && to.x == 2 && to.y == 0 => {
                let rook = self.take_stone_at(0, 0);
                self.stones[0][3] = rook;
                self.passant = None;
                self.castle_rules.black = CastleOptions::None;
                Move::Castle(CastlePosition::QueenSide)      
            }
            (stone, _, _) if stone.as_str() == "lk" => {
                self.passant = None;
                self.castle_rules.white = CastleOptions::None;
                Move::Normal
            }
            (stone, _, _) if stone.as_str() == "dk" => {
                self.passant = None;
                self.castle_rules.white = CastleOptions::None;
                Move::Normal
            }
            (stone, from, _) if stone.as_str() == "lr" && from.x == 0 && from.y == 7 => {
                self.passant = None;
                match self.castle_rules.white {
                    CastleOptions::BothSides => self.castle_rules.white = CastleOptions::KingSide,
                    CastleOptions::QueenSide => self.castle_rules.white = CastleOptions::None,
                    _ => {} 
                }
                Move::Normal
            }
            (stone, from, _) if stone.as_str() == "lr" && from.x == 7 && from.y == 7 => {
                self.passant = None;
                match self.castle_rules.white {
                    CastleOptions::BothSides => self.castle_rules.white = CastleOptions::QueenSide,
                    CastleOptions::KingSide => self.castle_rules.white = CastleOptions::None,
                    _ => {} 
                }
                Move::Normal
            }
            (stone, from, _) if stone.as_str() == "dr" && from.x == 0 && from.y == 0 => {
                self.passant = None;
                match self.castle_rules.black {
                    CastleOptions::BothSides => self.castle_rules.black = CastleOptions::KingSide,
                    CastleOptions::QueenSide => self.castle_rules.black = CastleOptions::None,
                    _ => {} 
                }
                Move::Normal
            }
            (stone, from, _) if stone.as_str() == "dr" && from.x == 7 && from.y == 0 => {
                self.passant = None;
                match self.castle_rules.black {
                    CastleOptions::BothSides => self.castle_rules.black = CastleOptions::QueenSide,
                    CastleOptions::KingSide => self.castle_rules.black = CastleOptions::None,
                    _ => {} 
                }
                Move::Normal
            }
            _ => {
                self.passant = None;
                Move::Normal
            }
        }
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
        let mut fen_split = fen_clone.split(" ").skip(4);
        let fen_castle_rules = self.castle_rules.to_string();
        new_fen.push_str(&format!(" {}", fen_castle_rules));
        let fen_passant = self.passant.as_ref().map(|pos| pos.to_string()).unwrap_or("-".to_string());
        new_fen.push_str(&format!(" {}", fen_passant));
        let fen_half_moves = fen_split.next().unwrap();
        new_fen.push_str(&format!(" {}", fen_half_moves));
        let fen_full_moves = fen_split.next().unwrap();
        new_fen.push_str(&format!(" {}", fen_full_moves));

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chess_board() {
        let fen = "rnb1kbnr/ppp1p1pp/8/4Pp2/8/3P1N2/PP1P1PPP/RNBQK2R w KQkq f6 0 1"; 
        let chess_board = ChessBoardBuilder::new().fen(fen).validation(true).sync(true).build().unwrap();

        assert_eq!(
            HashSet::<Position>::new(),
            chess_board.possible_moves(&Position::new(3, 2))
        );
    }
}