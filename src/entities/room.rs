use std::{collections::BTreeMap, str::FromStr};

use leptos::{create_rw_signal, RwSignal};

#[derive(Clone)]
pub struct RoomStatus {
    name: String,
    users: BTreeMap<String, RwSignal<User>>,
    options: ChessBoardOptions,
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

impl FromStr for User {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.splitn(3, ":");

        let Some(id) = split.next() else {
            return Err(());
        };

        let Some(username) = split.next() else {
            return Err(());
        };

        let Some(status) = split.next() else {
            return Err(());
        };

        let status = match status {
            "online" => UserStatus::Online,
            "offline" => UserStatus::Offline,
            "away" => UserStatus::Away,
            _ => return Err(()),
        };

        Ok(Self {
            id: id.to_string(),
            username: username.to_string(),
            status,
        })
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
        }
    }

    pub fn options(&self) -> &ChessBoardOptions {
        &self.options
    }

    pub fn options_string(&self) -> String {
        let mut options = String::new();

        if self.options.validation {
            options.push_str(" validation");
        }

        if self.options.sync {
            options.push_str(" sync");
        }

        options.trim().to_string()
    }

    pub fn set_options_from_str(&mut self, options: &str) {
        let mut options = options.split_whitespace();

        let mut validation = false;
        let mut sync = false;

        while let Some(option) = options.next() {
            match option {
                "validation" => validation = true,
                "sync" => sync = true,
                _ => (),
            }
        }

        if validation {
            self.enable_validation();
        } else {
            self.disable_validation();
        }

        if sync {
            self.enable_sync();
        } else {
            self.disable_sync();
        }
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

    pub fn sync_users(&mut self, users: Vec<String>) {
        self.users = users
            .into_iter()
            .map(|user| {
                let user = user.parse::<User>().unwrap();
                if let Some(old_user) = self.users.remove(&user.id()) {
                    (user.id(), old_user)
                } else {
                    (user.id(), create_rw_signal(user))
                }
            })
            .collect();
    }

    pub fn get_user(&self, id: &str) -> Option<RwSignal<User>> {
        self.users.get(id).cloned()
    }

    pub fn add_user(&mut self, user: User) {
        self.users.insert(user.id(), create_rw_signal(user));
    }

    pub fn remove_user(&mut self, username: &str) {
        if let Ok(user) = username.parse::<User>() {
            self.users.remove(&user.id());
        };
    }
}
