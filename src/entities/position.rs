use std::str::FromStr;

/// What the websocket protocol sends instead of a square for a stone in the trash
pub const DELETED: &str = "deleted";

#[derive(Clone, Debug, Eq, Hash)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Position {
        Position { x, y }
    }

    /// Parses a square, or [`DELETED`] for the trash
    pub fn parse_square(s: &str) -> Result<Option<Position>, ()> {
        if s == DELETED {
            Ok(None)
        } else {
            s.parse().map(Some)
        }
    }

    pub fn from_ui_position(ui_x: f64, ui_y: f64, is_white_view: bool) -> Position {
        let (x, y) = match is_white_view {
            true => (ui_x, ui_y),
            false => (800.0 - ui_x, 800.0 - ui_y),
        };

        let mut x = (x / 100.0) as i32;
        let mut y = (y / 100.0) as i32;

        if x < 0 {
            x = 0;
        } else if x > 7 {
            x = 7;
        }

        if y < 0 {
            y = 0;
        } else if y > 7 {
            y = 7;
        }

        Position::new(x as usize, y as usize)
    }

    pub fn to_string(&self) -> String {
        let buffer: [u8; 1] = [self.x as u8 + 97];
        format!("{}{}", std::str::from_utf8(&buffer).unwrap(), 8 - self.y)
    }

    pub fn x(&self) -> usize {
        self.x
    }

    pub fn y(&self) -> usize {
        self.y
    }

    // TODO: use this or remove it
    // pub fn set_ui_position(& mut self, is_white_view: bool) {
    //   let (ui_x, ui_y) = (self.x as f64 * 100.0, self.y as f64 * 100.0);

    //   match is_white_view {
    //     true => {
    //       self.ui_x = Some(ui_x);
    //       self.ui_y = Some(ui_y);
    //     },
    //     false => {
    //       self.ui_x = Some(800.0 - ui_x);
    //       self.ui_y = Some(800.0 - ui_y);
    //     }
    //   }
    // }
}

impl FromStr for Position {
    type Err = ();

    /// Parses a square like `e4`; anything off the board is an error.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let &[file @ b'a'..=b'h', rank @ b'1'..=b'8'] = s.as_bytes() else {
            return Err(());
        };

        Ok(Position::new((file - b'a') as usize, (b'8' - rank) as usize))
    }
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_square() {
        assert_eq!("a8".parse(), Ok(Position::new(0, 0)));
        assert_eq!("h1".parse(), Ok(Position::new(7, 7)));
        assert_eq!(Position::parse_square(DELETED), Ok(None));

        for square in ["i1", "a9", "a0", "`1", "A1", "e", "e44", ""] {
            assert_eq!(Position::parse_square(square), Err(()), "{square}");
        }
    }
}
