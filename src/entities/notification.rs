use serde::{Deserialize, Serialize};

use super::protocol::Text;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotifyType {
    Error,
    Success,
    Warning,
}

/// The toast at the top of the page. Its text is a key the view translates.
#[derive(Clone)]
pub struct Notification {
    /// Bumped by every `show`, so the view restarts its animation even when
    /// the previous notification is still active
    pub id: u64,
    pub is_active: bool,
    pub text: Option<Text>,
    pub notify_type: NotifyType,
}

impl Notification {
    pub fn new() -> Self {
        Self {
            id: 0,
            is_active: false,
            text: None,
            notify_type: NotifyType::Success,
        }
    }

    pub fn show(&mut self, notify_type: NotifyType, text: Text) {
        self.id += 1;
        self.is_active = true;
        self.text = Some(text);
        self.notify_type = notify_type;
    }

    pub fn disable(&mut self) {
        self.is_active = false;
    }
}
