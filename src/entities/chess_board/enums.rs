use serde::{Deserialize, Serialize};

use crate::entities::{
    position::Position,
    stone::{Color, Stone},
};

// The error payloads are only read through `Debug`, in logs
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum ChessBoardError {
    InvalidFen(FenError),
    InvalidDeletedStones,
    InvalidMove(MoveError),
    BuildError,
}

#[derive(Clone, Debug)]
pub enum MoveError {
    NoStoneFound,
    InvalidMove,
}

#[derive(Clone, Debug)]
pub enum FenError {
    InvalidFormat,
    InvalidTurn,
    InvalidCastleRules,
    InvalidPassant,
    InvalidStones,
    InvalidHalfMoveClock,
    InvalidFullMoveClock,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Move {
    Normal,
    Promotion(PromotionKind),
    Castle(CastlePosition),
    /// Holds the square of the captured pawn
    Passant(Position),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromotionKind {
    #[default]
    Queen,
    Rook,
    Bishop,
    Knight,
}

impl PromotionKind {
    pub fn stone(self, color: Color) -> Stone {
        let c = match self {
            PromotionKind::Queen => 'q',
            PromotionKind::Rook => 'r',
            PromotionKind::Bishop => 'b',
            PromotionKind::Knight => 'n',
        };
        let c = match color {
            Color::Light => c.to_ascii_uppercase(),
            Color::Dark => c,
        };
        Stone::try_from(c).expect("promotion pieces are valid stones")
    }
}

#[derive(Clone, Debug)]
pub enum CastlePosition {
    KingSide,
    QueenSide,
}
