use crate::entities::{chess_board::enums::FenError, stone::Stone};

pub fn fen_to_stones(field: &str) -> Result<[[Option<Stone>; 8]; 8], FenError> {
    const INIT: Option<Stone> = None;
    const ROW: [Option<Stone>; 8] = [INIT; 8];
    let mut stones = [ROW; 8];

    for (y, row) in field.split("/").enumerate() {
        if y > 7 {
            return Err(FenError::InvalidStones);
        }

        let mut empty_squares = 0;

        for (x, c) in row.chars().enumerate() {
            if x > 7 {
                return Err(FenError::InvalidStones);
            }

            if c.is_digit(10) {
                let n = c.to_digit(10).ok_or(FenError::InvalidStones)?;
                let x = x + empty_squares as usize;
                empty_squares += n - 1;
                for i in 0..n {
                    let x = x + i as usize;
                    if x > 7 {
                        return Err(FenError::InvalidStones);
                    }
                    stones[y][x] = None;
                }
            } else {
                let x = x + empty_squares as usize;
                if x > 7 {
                    return Err(FenError::InvalidStones);
                }
                stones[y][x] = Some(Stone::try_from(c).map_err(|_| FenError::InvalidStones)?);
            }
        }
    }

    Ok(stones)
}
