use crate::entities::position::Position;

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

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum PromotionKind {
    Queen,
    Rook,
    Bishop,
    Knight,
}

#[derive(Clone, Debug)]
pub enum CastlePosition {
    KingSide,
    QueenSide,
}
