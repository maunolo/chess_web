//! Usernames are shown to every user in a room, so they are trimmed, kept to
//! a length and stripped of control characters. Room names are part of game
//! links (`/g/<room>`), so they keep to characters a URL path doesn't escape.

pub const MAX_USERNAME_LEN: usize = 24;
pub const MAX_ROOM_NAME_LEN: usize = 32;

pub fn sanitize_username(name: &str) -> Option<String> {
    let name: String = name.chars().filter(|c| !c.is_control()).collect();
    let name: String = name
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(MAX_USERNAME_LEN)
        .collect();
    let name = name.trim_end();

    (!name.is_empty()).then(|| name.to_string())
}

pub fn sanitize_room_name(name: &str) -> Option<String> {
    let name: String = name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(MAX_ROOM_NAME_LEN)
        .collect();

    (!name.is_empty()).then_some(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_username() {
        assert_eq!(sanitize_username("  John   Doe "), Some("John Doe".to_string()));
        assert_eq!(sanitize_username("Ann: the Rook"), Some("Ann: the Rook".to_string()));
        assert_eq!(sanitize_username("a\nb"), Some("ab".to_string()));
        assert_eq!(sanitize_username(" \t "), None);
        assert_eq!(sanitize_username(&"x".repeat(100)).unwrap().len(), MAX_USERNAME_LEN);
    }

    #[test]
    fn test_sanitize_room_name() {
        assert_eq!(sanitize_room_name("main"), Some("main".to_string()));
        assert_eq!(sanitize_room_name("a|b"), Some("ab".to_string()));
        assert_eq!(sanitize_room_name("my room-2_x"), Some("myroom-2_x".to_string()));
        assert_eq!(sanitize_room_name("café"), Some("caf".to_string()));
        assert_eq!(sanitize_room_name("|"), None);
        assert_eq!(sanitize_room_name(""), None);
    }
}
