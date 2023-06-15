//! `ChessServer` is an actor. It maintains list of connection client session.
//! And manages available rooms. Peers send messages to other peers in same
//! room through `ChessServer`.

use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};

/// Chat server sends this messages to session
#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

/// Message for chat server communications

/// New chat session is created
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>,
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

/// Send message to specific room
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    /// Id of the client session
    pub id: usize,
    /// Peer message
    pub msg: String,
    /// Match name
    pub match_name: String,
}

/// List of available rooms
pub struct ListMatches;

impl actix::Message for ListMatches {
    type Result = Vec<String>;
}

/// Join room, if room does not exists create new one.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    /// Client ID
    pub id: usize,

    /// Match name
    pub name: String,
}

/// `ChessServer` manages chat rooms and responsible for coordinating chat session.
///
/// Implementation is very naïve.
#[derive(Debug)]
pub struct ChessServer {
    sessions: HashMap<usize, Recipient<Message>>,
    matches: HashMap<String, HashSet<usize>>,
    rng: ThreadRng,
    visitor_count: Arc<AtomicUsize>,
}

impl ChessServer {
    pub fn new(visitor_count: Arc<AtomicUsize>) -> ChessServer {
        // default room
        let mut matches = HashMap::new();
        matches.insert("main".to_owned(), HashSet::new());

        ChessServer {
            sessions: HashMap::new(),
            matches,
            rng: rand::thread_rng(),
            visitor_count,
        }
    }
}

impl ChessServer {
    /// Send message to all users in the room
    fn send_message(&self, match_name: &str, message: &str, skip_id: usize) {
        if let Some(sessions) = self.matches.get(match_name) {
            for id in sessions {
                if *id != skip_id {
                    if let Some(addr) = self.sessions.get(id) {
                        addr.do_send(Message(message.to_owned()));
                    }
                }
            }
        }
    }
}

/// Make actor from `ChessServer`
impl Actor for ChessServer {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;
}

/// Handler for Connect message.
///
/// Register new session and assign unique id to this session
impl Handler<Connect> for ChessServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        log::info!("Someone joined");

        // notify all users in same room
        self.send_message("main", "Someone joined", 0);

        // register session with random id
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr);

        // auto join session to main room
        self.matches
            .entry("main".to_owned())
            .or_insert_with(HashSet::new)
            .insert(id);

        let count = self.visitor_count.fetch_add(1, Ordering::SeqCst);
        self.send_message("main", &format!("Total visitors {count}"), 0);

        // send id back
        id
    }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for ChessServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        log::info!("Someone disconnected");

        let mut rooms: Vec<String> = Vec::new();

        // remove address
        if self.sessions.remove(&msg.id).is_some() {
            // remove session from all rooms
            for (name, sessions) in &mut self.matches {
                if sessions.remove(&msg.id) {
                    rooms.push(name.to_owned());
                }
            }
        }
        // send message to other users
        for room in rooms {
            self.send_message(&room, "Someone disconnected", 0);
        }
    }
}

/// Handler for Message message.
impl Handler<ClientMessage> for ChessServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        self.send_message(&msg.match_name, msg.msg.as_str(), msg.id);
    }
}

/// Handler for `ListRooms` message.
impl Handler<ListMatches> for ChessServer {
    type Result = MessageResult<ListMatches>;

    fn handle(&mut self, _: ListMatches, _: &mut Context<Self>) -> Self::Result {
        let mut rooms = Vec::new();

        for key in self.matches.keys() {
            rooms.push(key.to_owned())
        }

        MessageResult(rooms)
    }
}

/// Join room, send disconnect message to old room
/// send join message to new room
impl Handler<Join> for ChessServer {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Context<Self>) {
        let Join { id, name } = msg;
        let mut matches = Vec::new();

        // remove session from all rooms
        for (n, sessions) in &mut self.matches {
            if sessions.remove(&id) {
                matches.push(n.to_owned());
            }
        }
        // send message to other users
        for match_name in matches {
            self.send_message(&match_name, "Someone disconnected", 0);
        }

        self.matches
            .entry(name.clone())
            .or_insert_with(HashSet::new)
            .insert(id);

        self.send_message(&name, "Someone connected", id);
    }
}
