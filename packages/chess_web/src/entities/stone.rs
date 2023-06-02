#[derive(Clone)]
pub struct Stone {
    pub c: String,
    pub color: String,
    pub name: String,
    pub image_class: String
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
            _ => "none"
        };
        let color = match c.as_str() {
            "p" | "r" | "n" | "b" | "q" | "k" => "Black",
            "P" | "R" | "N" | "B" | "Q" | "K" => "White",
            _ => "none"
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
            _ => "none"
        };

        if name == "none" || color == "none" || image_class == "none" {
            return None;
        }

        Some(Self {
            c,
            name: name.to_string(),
            color: color.to_string(),
            image_class: image_class.to_string()
        })
    }
}