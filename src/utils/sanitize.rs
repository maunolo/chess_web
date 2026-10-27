//! Usernames and room names are embedded in the plain-text websocket protocol
//! (`/add_user id:name:status`, `/sync_users room|user,user`,
//! `/sync_board room|fen|trash`), so they can't contain its separators. A
//! username also prefixes relayed chat lines, so it can't start with `/` or
//! clients would read the line as a server command.

const SEPARATORS: [char; 3] = ['|', ',', ':'];
pub const MAX_USERNAME_LEN: usize = 24;
pub const MAX_ROOM_NAME_LEN: usize = 32;

fn allowed(c: &char) -> bool {
    !c.is_control() && !SEPARATORS.contains(c)
}

pub fn sanitize_username(name: &str) -> Option<String> {
    let name: String = name.chars().filter(allowed).collect();
    let name = name.split_whitespace().collect::<Vec<_>>().join(" ");
    let name: String = name
        .trim_start_matches(|c: char| c == '/' || c.is_whitespace())
        .chars()
        .take(MAX_USERNAME_LEN)
        .collect();
    let name = name.trim_end();

    (!name.is_empty()).then(|| name.to_string())
}

pub fn sanitize_room_name(name: &str) -> Option<String> {
    let name: String = name
        .chars()
        .filter(|c| allowed(c) && !c.is_whitespace())
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
        assert_eq!(sanitize_username("a:b,c|d"), Some("abcd".to_string()));
        assert_eq!(
            sanitize_username("/notify error hi"),
            Some("notify error hi".to_string())
        );
        assert_eq!(sanitize_username("|/checkmate"), Some("checkmate".to_string()));
        assert_eq!(sanitize_username("a\nb"), Some("ab".to_string()));
        assert_eq!(sanitize_username(" / "), None);
        assert_eq!(sanitize_username(&"x".repeat(100)).unwrap().len(), MAX_USERNAME_LEN);
    }

    #[test]
    fn test_sanitize_room_name() {
        assert_eq!(sanitize_room_name("main"), Some("main".to_string()));
        assert_eq!(sanitize_room_name("a|b"), Some("ab".to_string()));
        assert_eq!(sanitize_room_name("|"), None);
        assert_eq!(sanitize_room_name(""), None);
    }
}
