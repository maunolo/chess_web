use std::str::FromStr;

#[derive(Clone)]
pub struct Stone {
    pub c: String,
    pub color: String,
    pub name: String,
    pub image_class: String,
}

impl Stone {
    pub fn from_char(c: char) -> Option<Self> {
        let c = c.to_string();
        let name = match c.to_uppercase().as_str() {
            "K" => "King",
            "Q" => "Queen",
            "R" => "Rook",
            "B" => "Bishop",
            "N" => "Knight",
            "P" => "Pawn",
            _ => "none",
        };
        let color = match c.as_str() {
            "p" | "r" | "n" | "b" | "q" | "k" => "Dark",
            "P" | "R" | "N" | "B" | "Q" | "K" => "Light",
            _ => "none",
        };
        let image_class = match c.as_str() {
            "p" => "dp",
            "r" => "dr",
            "n" => "dn",
            "b" => "db",
            "q" => "dq",
            "k" => "dk",
            "P" => "lp",
            "R" => "lr",
            "N" => "ln",
            "B" => "lb",
            "Q" => "lq",
            "K" => "lk",
            _ => "none",
        };

        if name == "none" || color == "none" || image_class == "none" {
            return None;
        }

        Some(Self {
            c,
            name: name.to_string(),
            color: color.to_string(),
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

        let color = match s.chars().nth(0).unwrap() {
            'd' => "Dark",
            'l' => "Light",
            _ => "none",
        };

        let mut c = s.chars().nth(1).unwrap().to_string();
        if color == "White" {
            c = c.to_uppercase();
        }

        let name = match s.chars().nth(1).unwrap() {
            'p' => "Pawn",
            'r' => "Rook",
            'n' => "Knight",
            'b' => "Bishop",
            'q' => "Queen",
            'k' => "King",
            _ => "none",
        };

        Ok(Self {
            c,
            name: name.to_string(),
            color: color.to_string(),
            image_class: s.to_string(),
        })
    }
}
