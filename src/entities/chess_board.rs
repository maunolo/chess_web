use super::position::Position;
use super::stone::Stone;

pub fn fen_to_passant(field: &str) -> Option<Position> {
    if field == "-" {
        return None;
    }

    let x = field.chars().nth(0).unwrap() as i32 - 97;
    let y = 8 - field.chars().nth(1).unwrap().to_digit(10).unwrap() as i32;
    Some(Position::new(x, y))
}

pub fn fen_to_castle_rules(field: &str) -> CastleRules {
    let mut castle_rules = CastleRules {
        white: vec![],
        black: vec![],
    };

    field.chars().for_each(|c| {
        if c == 'K' {
            castle_rules.white.push(CastleOptions::CanCastleKingside);
        } else if c == 'Q' {
            castle_rules.white.push(CastleOptions::CanCastleQueenside);
        } else if c == 'k' {
            castle_rules.black.push(CastleOptions::CanCastleKingside);
        } else if c == 'q' {
            castle_rules.black.push(CastleOptions::CanCastleQueenside);
        }
    });

    castle_rules
}

pub fn fen_to_stones(field: &str) -> [[Option<Stone>; 8]; 8] {
    const INIT: Option<Stone> = None;
    const ROW: [Option<Stone>; 8] = [INIT; 8];
    let mut stones = [ROW; 8];

    field.split("/").enumerate().for_each(|(y, row)| {
        row.chars()
            .flat_map(|c| {
                if c.is_digit(10) {
                    let n = c.to_digit(10).unwrap();
                    (0..n).map(|_| '_').collect::<Vec<char>>()
                } else {
                    vec![c]
                }
            })
            .enumerate()
            .for_each(|(x, c)| match Stone::from_char(c) {
                Some(stone) => {
                    stones[y][x] = Some(stone);
                }
                _ => {}
            })
    });

    stones
}

#[derive(Clone)]
pub enum CastleOptions {
    CanCastleKingside,
    CanCastleQueenside,
}

#[derive(Clone)]
pub struct CastleRules {
    white: Vec<CastleOptions>,
    black: Vec<CastleOptions>,
}

#[derive(Clone)]
pub struct ChessBoard {
    pub fen: String,
    pub stones: [[Option<Stone>; 8]; 8],
    pub turn: String,
    pub castle_rules: CastleRules,
    pub passant: Option<Position>,
    pub half_move_clock: i32,
    pub full_move_clock: i32,
    pub deleted_stones: Vec<Stone>,
    pub is_white_view: bool,
    pub reset_count: usize,
}

#[allow(dead_code)]
impl ChessBoard {
    pub fn new(fen: &str) -> Self {
        let fen_fields = fen.split(" ").collect::<Vec<&str>>();
        Self {
            fen: fen.to_string(),
            stones: fen_to_stones(fen_fields[0]),
            turn: fen_fields[1].to_string(),
            castle_rules: fen_to_castle_rules(fen_fields[2]),
            passant: fen_to_passant(fen_fields[3]),
            half_move_clock: fen_fields[4].parse::<i32>().unwrap(),
            full_move_clock: fen_fields[5].parse::<i32>().unwrap(),
            deleted_stones: Vec::new(),
            is_white_view: true,
            reset_count: 0,
        }
    }

    // TODO: use this or remove it
    //   pub fn get(&self, position: &Position) -> Option<Stone> {
    //       self.stones[position.y as usize][position.x as usize].clone()
    //   }

    pub fn stones_and_positions(&self) -> Vec<(Position, Stone)> {
        self.stones
            .iter()
            .enumerate()
            .flat_map(|(y, row)| {
                let row_pieces: Vec<(Position, Stone)> = row
                    .iter()
                    .enumerate()
                    .filter_map(|(x, stone)| {
                        let stone = stone.clone()?;
                        let position = Position::new(x as i32, y as i32);
                        Some((position, stone))
                    })
                    .collect();
                row_pieces
            })
            .collect()
    }

    pub fn deleted_stones(&self) -> Vec<Stone> {
        self.deleted_stones.clone()
    }

    pub fn set_reset_count(&mut self, count: usize) {
        self.reset_count = count;
    }

    pub fn reset_count(&self) -> usize {
        self.reset_count
    }

    pub fn flip(&mut self) {
        self.is_white_view = !self.is_white_view;
    }

    pub fn trash_string(&self) -> String {
        let keys = self
            .deleted_stones
            .iter()
            .map(|s| s.image_class.clone())
            .collect::<Vec<String>>();
        keys.join(",")
    }

    pub fn set_trash_from_str(&mut self, trash: &str) {
        let keys = trash.split(",");
        for key in keys {
            if key == "" {
                continue;
            }
            let stone = key.parse::<Stone>().unwrap();
            self.deleted_stones.push(stone);
        }
    }

    pub fn move_piece(&mut self, piece: &str, from: Option<Position>, to: Option<Position>) {
        if from == to {
            return;
        }

        if from.is_none() {
            if let Some(stone_idx) = self
                .deleted_stones
                .iter()
                .position(|s| s.image_class == piece)
            {
                let stone = self.deleted_stones.remove(stone_idx);
                let to = to.unwrap();
                let old_piece = self.stones[to.y as usize][to.x as usize].take();
                self.stones[to.y as usize][to.x as usize] = Some(stone);
                if let Some(old_piece) = old_piece {
                    self.deleted_stones.push(old_piece);
                }
            };
        } else if to.is_none() {
            let from = from.unwrap();
            let stone = self.stones[from.y as usize][from.x as usize].take();
            if let Some(stone) = stone {
                self.deleted_stones.push(stone);
            }
        } else {
            let from = from.unwrap();
            let to = to.unwrap();
            let old_piece = self.stones[to.y as usize][to.x as usize].take();
            let stone = self.stones[from.y as usize][from.x as usize].take();
            self.stones[to.y as usize][to.x as usize] = stone;
            if let Some(old_piece) = old_piece {
                self.deleted_stones.push(old_piece);
            }
        }

        self.sync_fen();
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
                    new_fen.push_str(&stone.c.clone());
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
        let fen_clone = self.fen.clone();
        let fen_split = fen_clone.split(" ").skip(1);
        let fen_rest = fen_split.collect::<Vec<&str>>().join(" ");
        new_fen.push_str(&format!(" {}", fen_rest));

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

// impl PartialEq<ChessBoard> for ChessBoard {
//     fn eq(&self, other: &ChessBoard) -> bool {
//         self.fen == other.fen
//     }

//     fn ne(&self, other: &ChessBoard) -> bool {
//         self.fen != other.fen
//     }
// }
