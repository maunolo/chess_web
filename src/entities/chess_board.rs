use std::collections::HashSet;

use crate::entities::chess_board::enums::{CastlePosition, PromotionKind};

use self::castle_rules::{fen_to_castle_rules, CastleOptions, CastleRules};
use self::enums::{ChessBoardError, FenError, Move, MoveError};
use self::passants::fen_to_passant;
use self::stones::fen_to_stones;
use self::turns::{fen_to_turn, Turn};

use super::position::Position;
use super::stone::{Color, Kind, Stone};

pub mod castle_rules;
pub mod enums;
pub mod passants;
pub mod signals;
pub mod stones;
pub mod turns;

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
            deleted_stones
                .push(Stone::try_from(c).map_err(|_| ChessBoardError::InvalidDeletedStones)?);
        }

        let mut chess_board = ChessBoard {
            fen: fen.to_string(),
            stones: fen_to_stones(fen_fields[0]).map_err(|e| ChessBoardError::InvalidFen(e))?,
            threat_map: [[false; 8]; 8],
            turn: fen_to_turn(fen_fields[1]).map_err(|e| ChessBoardError::InvalidFen(e))?,
            castle_rules: fen_to_castle_rules(fen_fields[2])
                .map_err(|e| ChessBoardError::InvalidFen(e))?,
            passant: fen_to_passant(fen_fields[3]).map_err(|e| ChessBoardError::InvalidFen(e))?,
            half_move_clock: fen_fields[4]
                .parse::<i32>()
                .map_err(|_| ChessBoardError::InvalidFen(FenError::InvalidHalfMoveClock))?,
            full_move_clock: fen_fields[5]
                .parse::<i32>()
                .map_err(|_| ChessBoardError::InvalidFen(FenError::InvalidFullMoveClock))?,
            deleted_stones,
            is_white_view,
            validation,
            sync,
        };

        if chess_board.sync {
            chess_board.sync_threat_map();
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
    pub threat_map: [[bool; 8]; 8],
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
        ChessBoardBuilder::new().fen(fen).validation(false).build()
    }

    /// The side to move is in check and has no legal move. A position with no
    /// legal move and no check is a stalemate, not a checkmate.
    pub fn is_checkmate(&self) -> bool {
        self.is_in_check() && !self.has_legal_move()
    }

    /// The side to move isn't in check and has no legal move
    pub fn is_stalemate(&self) -> bool {
        !self.is_in_check() && !self.has_legal_move()
    }

    /// Neither side can ever checkmate: kings alone, or a king and one bishop
    /// or knight against a lone king
    pub fn has_insufficient_material(&self) -> bool {
        let mut minor_pieces = 0;
        for (_, stone) in self.stones_and_positions_iter() {
            match stone.kind() {
                Kind::King => {}
                Kind::Bishop | Kind::Knight => minor_pieces += 1,
                _ => return false,
            }
        }
        minor_pieces <= 1
    }

    /// `color` has more than a lone king
    pub fn has_material_beyond_king(&self, color: Color) -> bool {
        self.stones_and_positions_iter()
            .any(|(_, stone)| stone.color() == color && stone.kind() != Kind::King)
    }

    pub fn has_legal_move(&self) -> bool {
        self.stones_and_positions_iter()
            .filter(|(_, stone)| stone.color() == self.turn_color())
            .any(|(position, _)| !self.possible_moves(&position).is_empty())
    }

    fn turn_color(&self) -> Color {
        match self.turn {
            Turn::White => Color::Light,
            Turn::Black => Color::Dark,
        }
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

        self.threat_at(position.x, position.y)
    }

    pub fn valid_castle_rules(&self) -> bool {
        let valid_for = |options: &CastleOptions, y: usize, color: Color| {
            let (king_side, queen_side) = match options {
                CastleOptions::KingSide => (true, false),
                CastleOptions::QueenSide => (false, true),
                CastleOptions::BothSides => (true, true),
                CastleOptions::None => return true,
            };

            self.stone_at_is(4, y, Kind::King, color)
                && (!king_side || self.stone_at_is(7, y, Kind::Rook, color))
                && (!queen_side || self.stone_at_is(0, y, Kind::Rook, color))
        };

        valid_for(self.castle_rules.white(), 7, Color::Light)
            && valid_for(self.castle_rules.black(), 0, Color::Dark)
    }

    pub fn stone_at_is(&self, x: usize, y: usize, kind: Kind, color: Color) -> bool {
        self.stone_at(x, y)
            .map(|s| s.kind() == kind && s.color() == color)
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

    pub fn set_threat(&mut self, x: usize, y: usize) {
        self.threat_map[y][x] = true;
    }

    pub fn threat_at(&self, x: usize, y: usize) -> bool {
        self.threat_map[y][x]
    }

    pub fn free_at(&self, x: usize, y: usize) -> bool {
        self.stone_at(x, y).is_none() && !self.threat_at(x, y)
    }

    pub fn sync_threat_map(&mut self) {
        self.threat_map = [[false; 8]; 8];

        let mut threat_moves = HashSet::new();

        let threatening_color = match self.turn {
            Turn::White => Color::Dark,
            Turn::Black => Color::Light,
        };

        for (position, stone) in self
            .stones_and_positions_iter()
            .filter(|(_, stone)| stone.color() == threatening_color)
        {
            for threat_pos in stone.threats(&position, &self) {
                threat_moves.insert(threat_pos);
            }
        }

        for position in threat_moves {
            self.set_threat(position.x, position.y);
        }
    }

    pub fn possible_moves(&self, position: &Position) -> HashSet<Position> {
        let Some(stone) = self.stone_at(position.x, position.y) else {
            return HashSet::new();
        };

        stone
            .possible_moves(position, &self)
            .into_iter()
            .filter(|possible_move| {
                let mut chess_board = self.clone();
                chess_board.sync = false;
                let _ = chess_board.apply_move(
                    stone.as_str(),
                    Some(position.clone()),
                    Some(possible_move.clone()),
                    true,
                    PromotionKind::Queen,
                );
                chess_board.sync_threat_map();
                !chess_board.is_in_check()
            })
            .collect()
    }

    pub fn move_piece(
        &mut self,
        piece: &str,
        from: Option<Position>,
        to: Option<Position>,
    ) -> Result<Move, ChessBoardError> {
        self.move_piece_promoting(piece, from, to, PromotionKind::Queen)
    }

    /// Like `move_piece`, a pawn reaching the last rank becomes `promotion`
    pub fn move_piece_promoting(
        &mut self,
        piece: &str,
        from: Option<Position>,
        to: Option<Position>,
        promotion: PromotionKind,
    ) -> Result<Move, ChessBoardError> {
        if from == to {
            return Ok(Move::Normal);
        }

        if self.validation {
            self.validate_move(piece, from.as_ref(), to.as_ref())?;
        }

        self.apply_move(piece, from, to, self.validation, promotion)
    }

    fn validate_move(
        &self,
        piece: &str,
        from: Option<&Position>,
        to: Option<&Position>,
    ) -> Result<(), ChessBoardError> {
        let invalid_move = ChessBoardError::InvalidMove(MoveError::InvalidMove);
        let stone = piece
            .parse::<Stone>()
            .map_err(|_| ChessBoardError::InvalidMove(MoveError::NoStoneFound))?;

        if stone.color() != self.turn_color() {
            return Err(invalid_move);
        }

        let (Some(from), Some(to)) = (from, to) else {
            return Err(invalid_move);
        };

        if !self.possible_moves(from).contains(to) {
            return Err(invalid_move);
        }

        Ok(())
    }

    /// Moves `piece` without checking the rules. `from` / `to` set to `None`
    /// mean the trash. The stone at `from` must be `piece`, so a client with a
    /// stale board can't move a different stone than the one it sees.
    fn apply_move(
        &mut self,
        piece: &str,
        from: Option<Position>,
        to: Option<Position>,
        with_effects: bool,
        promotion: PromotionKind,
    ) -> Result<Move, ChessBoardError> {
        let no_stone_found = || ChessBoardError::InvalidMove(MoveError::NoStoneFound);
        let off_board = from.is_none() || to.is_none();

        let stone = match from.as_ref() {
            Some(from) => {
                if self.stone_at(from.x, from.y).map(|s| s.as_str()) != Some(piece) {
                    return Err(no_stone_found());
                }
                self.take_stone_at(from.x, from.y).ok_or_else(no_stone_found)?
            }
            None => {
                let idx = self
                    .deleted_stones
                    .iter()
                    .position(|s| s.as_str() == piece)
                    .ok_or_else(no_stone_found)?;
                self.deleted_stones.remove(idx)
            }
        };

        let mut captured = false;
        let result = match (from, to) {
            (_, None) => {
                self.deleted_stones.push(stone.clone());
                Move::Normal
            }
            (from, Some(to)) => {
                if let Some(old_stone) = self.take_stone_at(to.x, to.y) {
                    self.deleted_stones.push(old_stone);
                    captured = true;
                }
                self.stones[to.y][to.x] = Some(stone.clone());

                match from {
                    Some(from) if with_effects => {
                        self.apply_move_validation_and_effects((&stone, &from, &to), promotion)
                    }
                    _ => {
                        self.passant = None;
                        Move::Normal
                    }
                }
            }
        };

        if self.sync {
            if captured || off_board || stone.kind() == Kind::Pawn {
                self.half_move_clock = 0;
            } else {
                self.half_move_clock += 1;
            }
            if self.turn == Turn::Black {
                self.full_move_clock += 1;
            }
            self.turn = !self.turn;
            self.sync_threat_map();
        }
        // The FEN is the room's saved position, so it follows the stones even
        // when turns aren't synced
        self.sync_fen();

        Ok(result)
    }

    pub fn apply_move_validation_and_effects(
        &mut self,
        stone_move: (&Stone, &Position, &Position),
        promotion: PromotionKind,
    ) -> Move {
        match stone_move {
            (stone, _, to) if stone.kind() == Kind::Pawn && self.passant.as_ref() == Some(to) => {
                self.passant = None;
                let (captured_y, opponent) = match stone.color() {
                    Color::Light => (to.y + 1, Color::Dark),
                    Color::Dark => (to.y.wrapping_sub(1), Color::Light),
                };

                if captured_y < 8 && self.stone_at_is(to.x, captured_y, Kind::Pawn, opponent) {
                    if let Some(captured) = self.take_stone_at(to.x, captured_y) {
                        self.deleted_stones.push(captured);
                    }
                    Move::Passant(Position::new(to.x, captured_y))
                } else {
                    Move::Normal
                }
            }
            (stone, from, to) if stone.as_str() == "lp" && from.y == 6 && to.y == 4 => {
                self.passant = Some(Position::new(to.x, 5));
                Move::Normal
            }
            (stone, from, to) if stone.as_str() == "dp" && from.y == 1 && to.y == 3 => {
                self.passant = Some(Position::new(to.x, 2));
                Move::Normal
            }
            (stone, _, to)
                if stone.kind() == Kind::Pawn
                    && to.y == if stone.color() == Color::Light { 0 } else { 7 } =>
            {
                self.passant = None;
                self.stones[to.y][to.x] = Some(promotion.stone(stone.color()));
                Move::Promotion(promotion)
            }
            (stone, from, to)
                if stone.as_str() == "lk"
                    && from.x == 4
                    && from.y == 7
                    && to.x == 6
                    && to.y == 7 =>
            {
                let rook = self.take_stone_at(7, 7);
                self.stones[7][5] = rook;
                self.passant = None;
                self.castle_rules.white = CastleOptions::None;
                Move::Castle(CastlePosition::KingSide)
            }
            (stone, from, to)
                if stone.as_str() == "lk"
                    && from.x == 4
                    && from.y == 7
                    && to.x == 2
                    && to.y == 7 =>
            {
                let rook = self.take_stone_at(0, 7);
                self.stones[7][3] = rook;
                self.passant = None;
                self.castle_rules.white = CastleOptions::None;
                Move::Castle(CastlePosition::QueenSide)
            }
            (stone, from, to)
                if stone.as_str() == "dk"
                    && from.x == 4
                    && from.y == 0
                    && to.x == 6
                    && to.y == 0 =>
            {
                let rook = self.take_stone_at(7, 0);
                self.stones[0][5] = rook;
                self.passant = None;
                self.castle_rules.black = CastleOptions::None;
                Move::Castle(CastlePosition::KingSide)
            }
            (stone, from, to)
                if stone.as_str() == "dk"
                    && from.x == 4
                    && from.y == 0
                    && to.x == 2
                    && to.y == 0 =>
            {
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
                self.castle_rules.black = CastleOptions::None;
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
        let placement = self
            .stones
            .iter()
            .map(|row| {
                let mut rank = String::new();
                let mut empty = 0;
                for stone in row {
                    match stone {
                        Some(stone) => {
                            if empty > 0 {
                                rank.push_str(&empty.to_string());
                                empty = 0;
                            }
                            rank.push(stone.char());
                        }
                        None => empty += 1,
                    }
                }
                if empty > 0 {
                    rank.push_str(&empty.to_string());
                }
                rank
            })
            .collect::<Vec<_>>()
            .join("/");
        let turn = match self.turn {
            Turn::White => "w",
            Turn::Black => "b",
        };
        let passant = self
            .passant
            .as_ref()
            .map(|pos| pos.to_string())
            .unwrap_or("-".to_string());

        self.fen = format!(
            "{} {} {} {} {} {}",
            placement,
            turn,
            self.castle_rules.to_string(),
            passant,
            self.half_move_clock,
            self.full_move_clock
        );
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
        let chess_board = ChessBoardBuilder::new()
            .fen(fen)
            .validation(true)
            .sync(true)
            .build()
            .unwrap();

        assert_eq!(
            HashSet::<Position>::new(),
            chess_board.possible_moves(&Position::new(3, 2))
        );
    }

    fn board(fen: &str, validation: bool) -> ChessBoard {
        ChessBoardBuilder::new()
            .fen(fen)
            .validation(validation)
            .sync(true)
            .build()
            .unwrap()
    }

    fn pos(s: &str) -> Option<Position> {
        s.parse().ok()
    }

    #[test]
    fn test_off_board_square_does_not_panic() {
        let mut chess_board = board(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            false,
        );

        assert!(chess_board.move_piece("lp", pos("i1"), pos("e4")).is_err());
        assert!(chess_board.move_piece("lp", pos("e2"), pos("a9")).is_ok());
    }

    #[test]
    fn test_invalid_fen_fields_are_rejected() {
        let fens = [
            "rnbqkbnr/pppppppp/8/8/0/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq A3 0 1",
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq e9 0 1",
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq e4 0 1",
        ];

        for fen in fens {
            assert!(
                ChessBoardBuilder::new().fen(fen).build().is_err(),
                "{fen}"
            );
        }
    }

    #[test]
    fn test_only_pawns_capture_en_passant() {
        let mut chess_board = board("4k3/8/8/5n2/4P3/8/8/4K3 b - e3 0 1", true);

        let result = chess_board.move_piece("dn", pos("f5"), pos("e3"));

        assert!(matches!(result, Ok(Move::Normal)));
        assert!(chess_board.stone_at(4, 4).is_some());
    }

    #[test]
    fn test_en_passant_without_pawn_does_not_panic() {
        let mut chess_board = board("4k3/8/8/3P4/8/8/8/4K3 w - e6 0 1", true);

        assert!(chess_board.move_piece("lp", pos("d5"), pos("e6")).is_ok());
    }

    #[test]
    fn test_castle_rules_validation_checks_piece_kind() {
        let result = ChessBoardBuilder::new()
            .fen("4k3/8/8/8/8/8/8/4Q2R w K - 0 1")
            .validation(true)
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_stalemate_is_not_checkmate() {
        assert!(!board("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1", true).is_checkmate());
        assert!(board("7k/6Q1/6K1/8/8/8/8/8 b - - 0 1", true).is_checkmate());
    }

    #[test]
    fn test_move_clocks_advance() {
        let mut chess_board = board(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            true,
        );

        chess_board.move_piece("lp", pos("e2"), pos("e4")).unwrap();
        assert!(chess_board.fen.ends_with("b KQkq e3 0 1"));

        chess_board.move_piece("dp", pos("e7"), pos("e5")).unwrap();
        assert!(chess_board.fen.ends_with("w KQkq e6 0 2"));

        chess_board.move_piece("ln", pos("g1"), pos("f3")).unwrap();
        assert!(chess_board.fen.ends_with("b KQkq - 1 2"));
    }
}
