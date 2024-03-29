use std::{collections::HashSet, str::FromStr};

use super::{
    chess_board::{castle_rules::CastleOptions, turns::Turn, ChessBoard},
    position::Position,
};

#[derive(Debug)]
pub struct MovePattern {
    patterns: Vec<(i32, i32)>,
    sliding: bool,
}

impl MovePattern {
    pub fn moves_for(&self, position: &Position, chess_board: &ChessBoard) -> HashSet<Position> {
        let mut moves = HashSet::new();

        for (move_x, move_y) in &self.patterns {
            let mut x = position.x as i32 + move_x;
            let mut y = position.y as i32 + move_y;
            let mut multiply = 1;

            while x >= 0 && x <= 7 && y >= 0 && y <= 7 {
                moves.insert(Position::new(x as usize, y as usize));

                if !self.sliding {
                    break;
                }

                if chess_board.stone_at(x as usize, y as usize).is_some() {
                    break;
                }

                multiply += 1;

                x = position.x as i32 + move_x * multiply;
                y = position.y as i32 + move_y * multiply;
            }
        }

        moves
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

impl Kind {
    pub fn move_pattern(&self, color: Color) -> MovePattern {
        match self {
            Kind::King => MovePattern {
                patterns: vec![
                    (1, 1),
                    (1, 0),
                    (1, -1),
                    (0, 1),
                    (0, -1),
                    (-1, 1),
                    (-1, 0),
                    (-1, -1),
                ],
                sliding: false,
            },
            Kind::Queen => MovePattern {
                patterns: vec![
                    (1, 1),
                    (1, 0),
                    (1, -1),
                    (0, 1),
                    (0, -1),
                    (-1, 1),
                    (-1, 0),
                    (-1, -1),
                ],
                sliding: true,
            },
            Kind::Rook => MovePattern {
                patterns: vec![(1, 0), (0, 1), (0, -1), (-1, 0)],
                sliding: true,
            },
            Kind::Bishop => MovePattern {
                patterns: vec![(1, 1), (1, -1), (-1, 1), (-1, -1)],
                sliding: true,
            },
            Kind::Knight => MovePattern {
                patterns: vec![
                    (2, 1),
                    (2, -1),
                    (-2, 1),
                    (-2, -1),
                    (1, 2),
                    (1, -2),
                    (-1, 2),
                    (-1, -2),
                ],
                sliding: false,
            },
            Kind::Pawn => match color {
                Color::Dark => MovePattern {
                    patterns: vec![(0, 1)],
                    sliding: false,
                },
                Color::Light => MovePattern {
                    patterns: vec![(0, -1)],
                    sliding: false,
                },
            },
        }
    }

    pub fn pawn_eat_pattern(&self, color: Color) -> MovePattern {
        match self {
            Kind::Pawn => match color {
                Color::Dark => MovePattern {
                    patterns: vec![(-1, 1), (1, 1)],
                    sliding: false,
                },
                Color::Light => MovePattern {
                    patterns: vec![(-1, -1), (1, -1)],
                    sliding: false,
                },
            },
            _ => panic!("pawn_eat_pattern called on non-pawn"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Color {
    Dark,
    Light,
}

impl Color {
    pub fn to_string(&self) -> String {
        match self {
            Color::Dark => "dark".to_string(),
            Color::Light => "light".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Stone {
    c: char,
    color: Color,
    kind: Kind,
    image_class: String,
}

#[allow(dead_code)]
impl Stone {
    pub fn char(&self) -> char {
        self.c
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }

    pub fn as_str(&self) -> &str {
        &self.image_class
    }

    pub fn image_class(&self) -> String {
        self.image_class.clone()
    }

    pub fn threats(&self, position: &Position, chess_board: &ChessBoard) -> HashSet<Position> {
        match self.kind {
            Kind::Pawn => {
                let eat_pattern = self.kind.pawn_eat_pattern(self.color);

                eat_pattern
                    .moves_for(position, chess_board)
                    .into_iter()
                    .collect()
            }
            _ => self.possible_moves(position, chess_board),
        }
    }

    pub fn possible_moves(
        &self,
        position: &Position,
        chess_board: &ChessBoard,
    ) -> HashSet<Position> {
        let move_pattern = self.kind.move_pattern(self.color);
        let mut moves = move_pattern.moves_for(position, chess_board);

        self.calculate_special_rules(position, chess_board, &mut moves);

        moves
            .into_iter()
            .filter(|pos| {
                chess_board
                    .stone_at(pos.x, pos.y)
                    .map(|s| s.color != self.color)
                    .unwrap_or(true)
            })
            .collect()
    }

    fn calculate_special_rules(
        &self,
        position: &Position,
        chess_board: &ChessBoard,
        moves: &mut HashSet<Position>,
    ) {
        match self.kind {
            Kind::Pawn => {
                let Some(move_pos) = moves.iter().next().cloned() else {
                    return;
                };

                if chess_board.stone_at(move_pos.x, move_pos.y).is_some() {
                    moves.remove(&move_pos);
                }

                let eat_pattern = self.kind.pawn_eat_pattern(self.color);

                for (move_x, move_y) in &eat_pattern.patterns {
                    let x = position.x as i32 + move_x;
                    let y = position.y as i32 + move_y;

                    if x >= 0 && x <= 7 && y >= 0 && y <= 7 {
                        if let Some(stone) = chess_board.stone_at(x as usize, y as usize) {
                            if stone.color != self.color {
                                moves.insert(Position::new(x as usize, y as usize));
                            }
                        } else if let Some(passant_position) = chess_board.passant.as_ref() {
                            let new_position = Position::new(x as usize, y as usize);
                            if new_position == *passant_position {
                                moves.insert(new_position);
                            }
                        }
                    }
                }

                match (self.color, position.x, position.y) {
                    (Color::Light, x, y) if y == 6 => {
                        let y = position.y - 2;

                        if chess_board.stone_at(x, y).is_none()
                            && chess_board.stone_at(x, y + 1).is_none()
                        {
                            moves.insert(Position::new(x, y));
                        }
                    }
                    (Color::Dark, x, y) if y == 1 => {
                        let y = position.y + 2;

                        if chess_board.stone_at(x, y).is_none()
                            && chess_board.stone_at(x, y - 1).is_none()
                        {
                            moves.insert(Position::new(x, y));
                        }
                    }
                    _ => {}
                }
            }
            Kind::King => {
                for pos in moves.clone() {
                    if chess_board.threat_at(pos.x, pos.y) {
                        moves.remove(&pos);
                    }
                }

                match (self.color, chess_board.turn, chess_board.is_in_check()) {
                    (Color::Light, turn, in_check) if !(turn == Turn::White && in_check) => {
                        match chess_board.castle_rules().white() {
                            CastleOptions::BothSides => {
                                // King side
                                if chess_board.free_at(5, 7) && chess_board.free_at(6, 7) {
                                    moves.insert(Position::new(6, 7));
                                }
                                // Queen side
                                if chess_board.free_at(1, 7)
                                    && chess_board.free_at(2, 7)
                                    && chess_board.free_at(3, 7)
                                {
                                    moves.insert(Position::new(2, 7));
                                }
                            }
                            CastleOptions::KingSide => {
                                if chess_board.free_at(5, 7) && chess_board.free_at(6, 7) {
                                    moves.insert(Position::new(6, 7));
                                }
                            }
                            CastleOptions::QueenSide => {
                                if chess_board.free_at(1, 7)
                                    && chess_board.free_at(2, 7)
                                    && chess_board.free_at(3, 7)
                                {
                                    moves.insert(Position::new(2, 7));
                                }
                            }
                            _ => {}
                        }
                    }
                    (Color::Dark, turn, in_check) if !(turn == Turn::Black && in_check) => {
                        match chess_board.castle_rules().black() {
                            CastleOptions::BothSides => {
                                // King side
                                if chess_board.free_at(5, 0) && chess_board.free_at(6, 0) {
                                    moves.insert(Position::new(6, 0));
                                }
                                // Queen side
                                if chess_board.free_at(1, 0)
                                    && chess_board.free_at(2, 0)
                                    && chess_board.free_at(3, 0)
                                {
                                    moves.insert(Position::new(2, 0));
                                }
                            }
                            CastleOptions::KingSide => {
                                if chess_board.free_at(5, 0) && chess_board.free_at(6, 0) {
                                    moves.insert(Position::new(6, 0));
                                }
                            }
                            CastleOptions::QueenSide => {
                                if chess_board.free_at(1, 0)
                                    && chess_board.free_at(2, 0)
                                    && chess_board.free_at(3, 0)
                                {
                                    moves.insert(Position::new(2, 0));
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

impl TryFrom<char> for Stone {
    type Error = ();

    fn try_from(c: char) -> Result<Self, Self::Error> {
        let kind = match c.to_uppercase().next().ok_or(())? {
            'K' => Kind::King,
            'Q' => Kind::Queen,
            'R' => Kind::Rook,
            'B' => Kind::Bishop,
            'N' => Kind::Knight,
            'P' => Kind::Pawn,
            _ => return Err(()),
        };
        let color = match c {
            'p' | 'r' | 'n' | 'b' | 'q' | 'k' => Color::Dark,
            'P' | 'R' | 'N' | 'B' | 'Q' | 'K' => Color::Light,
            _ => return Err(()),
        };
        let image_class = match c {
            'p' => "dp",
            'r' => "dr",
            'n' => "dn",
            'b' => "db",
            'q' => "dq",
            'k' => "dk",
            'P' => "lp",
            'R' => "lr",
            'N' => "ln",
            'B' => "lb",
            'Q' => "lq",
            'K' => "lk",
            _ => return Err(()),
        };

        Ok(Self {
            c,
            kind,
            color,
            image_class: image_class.to_string(),
        })
    }
}

impl FromStr for Stone {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            return Err(());
        }

        let color = match s.chars().nth(0).ok_or(())? {
            'd' => Color::Dark,
            'l' => Color::Light,
            _ => return Err(()),
        };

        let mut c = s.chars().nth(1).ok_or(())?;
        if matches!(color, Color::Light) {
            c = c.to_uppercase().nth(0).ok_or(())?;
        }

        let kind = match s.chars().nth(1).ok_or(())? {
            'p' => Kind::Pawn,
            'r' => Kind::Rook,
            'n' => Kind::Knight,
            'b' => Kind::Bishop,
            'q' => Kind::Queen,
            'k' => Kind::King,
            _ => return Err(()),
        };

        Ok(Self {
            c,
            kind,
            color,
            image_class: s.to_string(),
        })
    }
}
