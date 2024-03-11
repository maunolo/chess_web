use super::FenError;

#[derive(Clone, Debug)]
pub enum CastleOptions {
    KingSide,
    QueenSide,
    BothSides,
    None,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct CastleRules {
    pub white: CastleOptions,
    pub black: CastleOptions,
}

impl CastleRules {
    pub fn to_string(&self) -> String {
        let white = match self.white {
            CastleOptions::KingSide => "K",
            CastleOptions::QueenSide => "Q",
            CastleOptions::BothSides => "KQ",
            CastleOptions::None => "",
        };
        let black = match self.black {
            CastleOptions::KingSide => "k",
            CastleOptions::QueenSide => "q",
            CastleOptions::BothSides => "kq",
            CastleOptions::None => "",
        };
        let str = format!("{}{}", white, black);
        if str.is_empty() {
            "-".to_string()
        } else {
            str
        }
    }

    pub fn white(&self) -> &CastleOptions {
        &self.white
    }

    pub fn black(&self) -> &CastleOptions {
        &self.black
    }
}

pub fn fen_to_castle_rules(field: &str) -> Result<CastleRules, FenError> {
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
                return Err(FenError::InvalidCastleRules);
            }
        }
    }

    Ok(CastleRules {
        white: white_castle_options,
        black: black_castle_options,
    })
}
