use crate::entities::position::Position;

use super::enums::FenError;

pub fn fen_to_passant(field: &str) -> Result<Option<Position>, FenError> {
    if field == "-" {
        return Ok(None);
    }

    let x = field.chars().nth(0).ok_or(FenError::InvalidPassant)? as usize - 97;
    let y = 8 - field
        .chars()
        .nth(1)
        .ok_or(FenError::InvalidPassant)?
        .to_digit(10)
        .ok_or(FenError::InvalidPassant)? as usize;
    Ok(Some(Position::new(x, y)))
}
