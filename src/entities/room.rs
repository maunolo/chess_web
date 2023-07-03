use std::{collections::HashMap, str::FromStr};

#[derive(Clone)]
pub struct RoomStatus {
    name: String,
    users: HashMap<String, User>,
}

#[derive(Clone)]
pub struct User {
    id: String,
    username: String,
}

impl User {
    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn username(&self) -> String {
        self.username.clone()
    }
}

impl FromStr for User {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split = s.split_once(":").unwrap();

        Ok(Self {
            id: split.0.to_string(),
            username: split.1.to_string(),
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
                let split = user.split_once(":").unwrap();

                (
                    split.0.to_string(),
                    User {
                        id: split.0.to_string(),
                        username: split.1.to_string(),
                    },
                )
            })
            .collect();
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
}
