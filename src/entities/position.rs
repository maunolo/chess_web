use std::str::FromStr;

#[derive(Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub ui_x: Option<f64>,
    pub ui_y: Option<f64>,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Position {
        Position {
            x,
            y,
            ui_x: None,
            ui_y: None,
        }
    }

    pub fn from_ui_position(ui_x: f64, ui_y: f64, is_white_view: bool) -> Position {
        let (x, y) = match is_white_view {
            true => (ui_x, ui_y),
            false => (800.0 - ui_x, 800.0 - ui_y),
        };

        let x = (x / 100.0) as i32;
        let y = (y / 100.0) as i32;

        Position {
            x,
            y,
            ui_x: Some(ui_x),
            ui_y: Some(ui_y),
        }
    }

    pub fn to_string(&self) -> String {
        let buffer: [u8; 1] = [self.x as u8 + 97];
        format!("{}{}", std::str::from_utf8(&buffer).unwrap(), 8 - self.y)
    }

    pub fn css_class(&self) -> String {
        format!("square-{}", self.to_string())
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

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let x = s.as_bytes()[0] as i32 - 97;
        let y = 8 - s.as_bytes()[1] as i32 - 48;
        Ok(Position::new(x, y))
    }
}
