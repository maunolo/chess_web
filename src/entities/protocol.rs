//! Messages between the browser and the server: one JSON object per
//! websocket text frame, tagged by its `type` field.

use serde::{Deserialize, Serialize};

use super::{
    chess_board::{enums::PromotionKind, turns::Turn},
    notification::NotifyType,
};

/// A player's color in a game
pub type Side = Turn;

/// The room everyone lands in at `/`
pub const MAIN_ROOM: &str = "main";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMsg {
    /// Enters a room. A free board room that doesn't exist yet starts from
    /// `fen` and `trash`, or the initial position.
    Join {
        room: String,
        #[serde(default)]
        fen: Option<String>,
        #[serde(default)]
        trash: Option<String>,
    },
    Username {
        name: String,
    },
    /// `from` and `to` are squares like `e2`, or `deleted` for the trash
    Move {
        piece: String,
        from: String,
        to: String,
        #[serde(default)]
        promotion: Option<PromotionKind>,
    },
    Reset,
    Undo,
    Redo,
    Options {
        validation: bool,
        sync: bool,
    },
    Chat {
        text: String,
    },
    /// Creates a game the user is seated in, answered with `GameCreated`
    NewGame {
        color: SeatChoice,
        #[serde(default)]
        time_control: Option<TimeControl>,
    },
    Resign,
    OfferDraw,
    AcceptDraw,
    DeclineDraw,
    /// Offers a rematch, or accepts the opponent's offer
    Rematch,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMsg {
    SyncBoard {
        room: String,
        fen: String,
        trash: String,
    },
    SyncUsers {
        room: String,
        users: Vec<UserInfo>,
    },
    SyncOptions {
        validation: bool,
        sync: bool,
    },
    /// A user entered the room or changed their name
    UserJoined {
        user: UserInfo,
    },
    UserLeft {
        id: String,
    },
    UserPresence {
        id: String,
        presence: Presence,
    },
    /// Another user's move, or the side effect of a move (the pawn taken en
    /// passant, the castling rook). `fen` and `trash` are the board after the
    /// whole move, so clients take the turn, castling rights and en passant
    /// square from the server instead of tracking them.
    Move {
        piece: String,
        from: String,
        to: String,
        fen: String,
        trash: String,
    },
    /// The board after the user's own move, which isn't sent back to them
    Position {
        fen: String,
        trash: String,
    },
    Notify {
        level: NotifyType,
        message: String,
    },
    /// The side to move on a free board with validation is checkmated
    Checkmate,
    Chat {
        name: String,
        text: String,
    },
    /// The room's game, none on a free board
    Game {
        game: Option<GameInfo>,
    },
    GameCreated {
        room: String,
    },
    /// The game of a link ended a while ago and was cleaned up
    RoomGone {
        room: String,
    },
    /// The user connected again on another socket, which replaces this one
    Replaced,
    Error {
        message: String,
    },
}

impl ServerMsg {
    pub fn notify(level: NotifyType, message: impl Into<String>) -> Self {
        Self::Notify {
            level,
            message: message.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Presence {
    Online,
    Away,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub name: String,
    pub presence: Presence,
}

/// The seat the creator of a game takes
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeatChoice {
    White,
    Black,
    Random,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeControl {
    pub initial_secs: u32,
    pub increment_secs: u32,
}

impl TimeControl {
    pub fn label(&self) -> String {
        let minutes = self.initial_secs / 60;
        let initial = if self.initial_secs % 60 == 0 {
            minutes.to_string()
        } else {
            format!("{:.1}", self.initial_secs as f64 / 60.0)
        };
        format!("{}+{}", initial, self.increment_secs)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClockInfo {
    pub white_ms: i64,
    pub black_ms: i64,
    /// The side whose time is running: none before the first move and once
    /// the game is over
    pub running: Option<Side>,
}

impl ClockInfo {
    pub fn remaining_ms(&self, side: Side) -> i64 {
        match side {
            Side::White => self.white_ms,
            Side::Black => self.black_ms,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum GameResult {
    Checkmate { winner: Side },
    Resignation { winner: Side },
    Timeout { winner: Side },
    Stalemate,
    Agreement,
    InsufficientMaterial,
    /// A side ran out of time, but the other one has only its king left
    TimeoutVsInsufficientMaterial,
    Repetition,
    FiftyMoves,
}

impl GameResult {
    pub fn winner(&self) -> Option<Side> {
        match self {
            GameResult::Checkmate { winner }
            | GameResult::Resignation { winner }
            | GameResult::Timeout { winner } => Some(*winner),
            _ => None,
        }
    }

    pub fn description(&self) -> String {
        let side_name = |side: &Side| match side {
            Side::White => "White",
            Side::Black => "Black",
        };
        match self {
            GameResult::Checkmate { winner } => format!("{} wins by checkmate", side_name(winner)),
            GameResult::Resignation { winner } => {
                format!("{} wins by resignation", side_name(winner))
            }
            GameResult::Timeout { winner } => format!("{} wins on time", side_name(winner)),
            GameResult::Stalemate => "Draw by stalemate".to_string(),
            GameResult::Agreement => "Draw by agreement".to_string(),
            GameResult::InsufficientMaterial => "Draw by insufficient material".to_string(),
            GameResult::TimeoutVsInsufficientMaterial => {
                "Draw: time out with no mating material".to_string()
            }
            GameResult::Repetition => "Draw by threefold repetition".to_string(),
            GameResult::FiftyMoves => "Draw by the fifty-move rule".to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameInfo {
    /// User id seated as white
    pub white: Option<String>,
    /// User id seated as black
    pub black: Option<String>,
    pub time_control: Option<TimeControl>,
    pub clock: Option<ClockInfo>,
    pub result: Option<GameResult>,
    pub draw_offer: Option<Side>,
    pub rematch_offer: Option<Side>,
}

impl GameInfo {
    pub fn seat_of(&self, user_id: &str) -> Option<Side> {
        if self.white.as_deref() == Some(user_id) {
            Some(Side::White)
        } else if self.black.as_deref() == Some(user_id) {
            Some(Side::Black)
        } else {
            None
        }
    }

    pub fn player(&self, side: Side) -> Option<&str> {
        match side {
            Side::White => self.white.as_deref(),
            Side::Black => self.black.as_deref(),
        }
    }

    pub fn is_full(&self) -> bool {
        self.white.is_some() && self.black.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_messages_are_tagged_json() {
        let msg = ClientMsg::Move {
            piece: "lp".to_string(),
            from: "e2".to_string(),
            to: "e4".to_string(),
            promotion: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(
            json,
            r#"{"type":"move","piece":"lp","from":"e2","to":"e4","promotion":null}"#
        );
        assert_eq!(serde_json::from_str::<ClientMsg>(&json).unwrap(), msg);

        // optional fields may be left out
        let join: ClientMsg = serde_json::from_str(r#"{"type":"join","room":"main"}"#).unwrap();
        assert_eq!(
            join,
            ClientMsg::Join {
                room: "main".to_string(),
                fen: None,
                trash: None
            }
        );

        let result = ServerMsg::Game {
            game: Some(GameInfo {
                white: Some("a".to_string()),
                black: None,
                time_control: Some(TimeControl {
                    initial_secs: 300,
                    increment_secs: 3,
                }),
                clock: None,
                result: Some(GameResult::Timeout {
                    winner: Side::Black,
                }),
                draw_offer: None,
                rematch_offer: None,
            }),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains(r#""result":{"reason":"timeout","winner":"black"}"#));
        assert_eq!(serde_json::from_str::<ServerMsg>(&json).unwrap(), result);
    }

    #[test]
    fn test_time_control_label() {
        let label = |initial_secs, increment_secs| {
            TimeControl {
                initial_secs,
                increment_secs,
            }
            .label()
        };
        assert_eq!(label(300, 3), "5+3");
        assert_eq!(label(30, 0), "0.5+0");
    }
}
