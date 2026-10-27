use std::collections::BTreeMap;

use leptos::prelude::*;

use super::protocol::{Presence, UserInfo};

#[derive(Clone)]
pub struct RoomStatus {
    name: String,
    users: BTreeMap<String, RwSignal<User>>,
    options: ChessBoardOptions,
    checkmate: bool,
}

#[derive(Clone)]
pub struct ChessBoardOptions {
    validation: bool,
    sync: bool,
}

impl ChessBoardOptions {
    pub fn validation(&self) -> bool {
        self.validation
    }

    pub fn sync(&self) -> bool {
        self.sync
    }
}

#[derive(Clone)]
pub enum UserStatus {
    Online,
    Offline,
    Away,
}

#[derive(Clone)]
pub struct User {
    id: String,
    username: String,
    status: UserStatus,
}

#[allow(dead_code)]
impl User {
    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn username(&self) -> String {
        self.username.clone()
    }

    pub fn status(&self) -> UserStatus {
        self.status.clone()
    }

    pub fn status_str(&self) -> String {
        match self.status {
            UserStatus::Online => "online".to_string(),
            UserStatus::Offline => "offline".to_string(),
            UserStatus::Away => "away".to_string(),
        }
    }

    pub fn set_username(&mut self, username: &str) {
        if self.username != username {
            self.username = username.to_string();
        }
    }

    pub fn disconnect(&mut self) {
        self.status = UserStatus::Away;
    }

    pub fn connect(&mut self) {
        self.status = UserStatus::Online;
    }

    pub fn logout(&mut self) {
        self.status = UserStatus::Offline;
    }
}

impl From<UserInfo> for User {
    fn from(info: UserInfo) -> Self {
        Self {
            id: info.id,
            username: info.name,
            status: match info.presence {
                Presence::Online => UserStatus::Online,
                Presence::Away => UserStatus::Away,
            },
        }
    }
}

#[allow(dead_code)]
impl RoomStatus {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            users: BTreeMap::new(),
            options: ChessBoardOptions {
                validation: false,
                sync: true,
            },
            checkmate: false,
        }
    }

    pub fn options(&self) -> &ChessBoardOptions {
        &self.options
    }

    pub fn set_options(&mut self, validation: bool, sync: bool) {
        self.options.validation = validation;
        self.options.sync = sync;
    }

    pub fn enable_validation(&mut self) {
        self.options.validation = true;
    }

    pub fn disable_validation(&mut self) {
        self.options.validation = false;
    }

    pub fn enable_sync(&mut self) {
        self.options.sync = true;
    }

    pub fn disable_sync(&mut self) {
        self.options.sync = false;
    }

    pub fn users_count(&self) -> usize {
        self.users.values().len()
    }

    pub fn users(&self) -> Vec<RwSignal<User>> {
        self.users.values().cloned().collect()
    }

    pub fn users_map(self) -> BTreeMap<String, RwSignal<User>> {
        self.users
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn set_name(&mut self, name: &str) {
        if self.name != name {
            self.name = name.to_string();
        }
    }

    pub fn sync_users(&mut self, users: Vec<UserInfo>) {
        self.users = users
            .into_iter()
            .map(User::from)
            .map(|user| {
                if let Some(old_user) = self.users.remove(&user.id()) {
                    (user.id(), old_user)
                } else {
                    (user.id(), RwSignal::new(user))
                }
            })
            .collect();
    }

    pub fn get_user(&self, id: &str) -> Option<RwSignal<User>> {
        self.users.get(id).cloned()
    }

    pub fn add_user(&mut self, user: User) {
        self.users.insert(user.id(), RwSignal::new(user));
    }

    pub fn remove_user(&mut self, id: &str) {
        self.users.remove(id);
    }

    pub fn checkmate(&self) -> bool {
        self.checkmate
    }

    pub fn set_checkmate(&mut self, checkmate: bool) {
        self.checkmate = checkmate;
    }
}
