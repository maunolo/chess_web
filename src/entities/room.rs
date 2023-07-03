use std::{collections::HashMap, str::FromStr};

#[derive(Clone)]
pub struct RoomStatus {
    name: String,
    users: HashMap<String, User>,
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

impl User {
    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn username(&self) -> String {
        self.username.clone()
    }

    pub fn status(&self) -> String {
        match self.status {
            UserStatus::Online => "online".to_string(),
            UserStatus::Offline => "offline".to_string(),
            UserStatus::Away => "away".to_string(),
        }
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
            users: HashMap::new(),
        }
    }

    pub fn users_count(&self) -> usize {
        self.users.len()
    }

    pub fn users(&self) -> Vec<User> {
        self.users.values().cloned().collect()
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
            .iter()
            .map(|user| {
                let user = user.parse::<User>().unwrap();

                (user.id.to_string(), user)
            })
            .collect();
    }

    pub fn set_all_users_offline(&mut self) {
        for user in self.users.values_mut() {
            user.status = UserStatus::Offline;
        }
    }

    pub fn add_user(&mut self, username: &str) {
        if let Ok(user) = username.parse::<User>() {
            self.users.insert(user.id(), user);
        };
    }

    pub fn remove_user(&mut self, username: &str) {
        if let Ok(user) = username.parse::<User>() {
            self.users.remove(&user.id());
        };
    }

    pub fn disconnect_user(&mut self, username: &str) {
        if let Ok(user) = username.parse::<User>() {
            if let Some(user) = self.users.get_mut(&user.id()) {
                user.status = UserStatus::Away;
            }
        };
    }

    pub fn connect_user(&mut self, username: &str) {
        if let Ok(user) = username.parse::<User>() {
            if let Some(user) = self.users.get_mut(&user.id()) {
                user.status = UserStatus::Online;
            }
        };
    }
}
