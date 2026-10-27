use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotifyType {
    Error,
    Success,
    Warning,
}

#[derive(Clone)]
pub struct Notification {
    /// Bumped by every `show`, so the view restarts its animation even when
    /// the previous notification is still active
    pub id: u64,
    pub is_active: bool,
    pub message: String,
    pub notify_type: NotifyType,
}

#[allow(dead_code)]
impl Notification {
    pub fn new(message: String, notify_type: NotifyType) -> Self {
        Self {
            id: 0,
            is_active: false,
            message,
            notify_type,
        }
    }

    pub fn show(&mut self, notify_type: NotifyType, message: &str) {
        self.id += 1;
        self.is_active = true;
        self.message = message.to_string();
        self.notify_type = notify_type;
    }

    pub fn disable(&mut self) {
        self.is_active = false;
    }

    pub fn enable(&mut self) {
        self.is_active = true;
    }
}
