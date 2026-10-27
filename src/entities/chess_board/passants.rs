use crate::entities::position::Position;

use super::enums::FenError;

pub fn fen_to_passant(field: &str) -> Result<Option<Position>, FenError> {
    if field == "-" {
        return Ok(None);
    }

    // An en passant target square can only be on the 3rd or 6th rank
    match field.parse::<Position>() {
        Ok(position) if position.y == 2 || position.y == 5 => Ok(Some(position)),
        _ => Err(FenError::InvalidPassant),
    }
}
