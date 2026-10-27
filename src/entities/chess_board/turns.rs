use std::ops::Not;

use serde::{Deserialize, Serialize};

use super::enums::FenError;

pub fn fen_to_turn(field: &str) -> Result<Turn, FenError> {
    if field == "w" {
        Ok(Turn::White)
    } else if field == "b" {
        Ok(Turn::Black)
    } else {
        Err(FenError::InvalidTurn)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
