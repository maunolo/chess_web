#[derive(Clone)]
pub struct RoomStatus {
    name: String,
    users: Vec<String>,
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

#[allow(dead_code)]
impl RoomStatus {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            users: vec![],
        }
    }

    pub fn users_count(&self) -> usize {
        self.users.len()
    }

    pub fn users(&self) -> Vec<User> {
        self.users
            .iter()
            .map(|user| {
                let split = user.split_once(":").unwrap();

                User {
                    id: split.0.to_string(),
                    username: split.1.to_string(),
                }
            })
            .collect()
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
        self.users = users;
    }

    pub fn add_user(&mut self, username: &str) {
        self.users.push(username.to_string());
    }

    pub fn remove_user(&mut self, username: &str) {
        self.users.retain(|u| u != username);
    }
}
