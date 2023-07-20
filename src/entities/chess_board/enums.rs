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

#[derive(Clone, Debug)]
pub enum Move {
    Normal,
    Promotion(PromotionKind),
    Castle(CastlePosition),
    Passant,
}

#[derive(Clone, Debug)]
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
