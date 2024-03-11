#[derive(Clone)]
#[allow(dead_code)]
pub enum NotifyType {
    Error,
    Success,
    Warning,
}

#[derive(Clone)]
pub struct Notification {
    pub is_active: bool,
    pub message: String,
    pub notify_type: NotifyType,
}

#[allow(dead_code)]
impl Notification {
    pub fn new(message: String, notify_type: NotifyType) -> Self {
        Self {
            is_active: false,
            message,
            notify_type,
        }
    }

    pub fn disable(&mut self) {
        self.is_active = false;
    }

    pub fn enable(&mut self) {
        self.is_active = true;
    }
}
