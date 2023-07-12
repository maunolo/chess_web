use std::{collections::HashSet, str::FromStr};

use super::{
    chess_board::{CastleOptions, ChessBoard},
    position::Position,
};

#[derive(Debug)]
pub struct MovePattern {
    patterns: Vec<(i32, i32)>,
    sliding: bool,
}

impl MovePattern {
    pub fn moves_for(&self, position: &Position, chess_board: &ChessBoard) -> Vec<Position> {
        let mut moves = Vec::new();

        for (move_x, move_y) in &self.patterns {
            let mut x = position.x as i32 + move_x;
            let mut y = position.y as i32 + move_y;
            let mut multiply = 1;

            while x >= 0 && x <= 7 && y >= 0 && y <= 7 {
                moves.push(Position::new(x as usize, y as usize));

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

    pub fn image_class(&self) -> String {
        self.image_class.clone()
    }

    pub fn possible_moves(
        &self,
        position: &Position,
        chess_board: &ChessBoard,
    ) -> HashSet<Position> {
        let mut moves = HashSet::<Position>::new();
        let move_pattern = self.kind.move_pattern(self.color);

        for move_pos in move_pattern.moves_for(position, chess_board) {
            moves.insert(move_pos);
        }
        match self.kind {
            Kind::Pawn => {
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

                match self.color {
                    Color::Light => {
                        if position.y == 6 {
                            let x = position.x as i32;
                            let y = position.y as i32 - 2;

                            if chess_board.stone_at(x as usize, y as usize).is_none()
                                && chess_board.stone_at(x as usize, y as usize + 1).is_none()
                            {
                                moves.insert(Position::new(x as usize, y as usize));
                            }
                        }
                    }
                    Color::Dark => {
                        if position.y == 1 {
                            let x = position.x as i32;
                            let y = position.y as i32 + 2;

                            if chess_board.stone_at(x as usize, y as usize).is_none()
                                && chess_board.stone_at(x as usize, y as usize - 1).is_none()
                            {
                                moves.insert(Position::new(x as usize, y as usize));
                            }
                        }
                    }
                }
            }
            Kind::King => {
                for move_pos in moves.clone() {
                    if chess_board.treat_at(move_pos.x, move_pos.y) {
                        moves.remove(&move_pos);
                    }
                }

                match self.color {
                    Color::Light => {
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
                    Color::Dark => {
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
                }
            }
            _ => {}
        }

        for move_pos in moves.clone() {
            if let Some(stone) = chess_board.stone_at(move_pos.x, move_pos.y) {
                if stone.color == self.color {
                    moves.remove(&move_pos);
                }
            }
        }

        moves
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
