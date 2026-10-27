use std::cell::Cell;

use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::{CloseEvent, Element, ErrorEvent, MessageEvent, Response, WebSocket};

use crate::{
    entities::{
        chess_board::{
            signals::{ChessBoardSignals, Connection},
            ChessBoardBuilder,
        },
        notification::NotifyType,
        position::{Position, DELETED},
        protocol::{Presence, ServerMsg, MAIN_ROOM},
        room::{RoomStatus, User},
    },
    utils::{
        browser::{expire_cookie, now_ms},
        class_list::ClassListExt,
        elements::document,
        WindowExt,
    },
};

const INITIAL_RECONNECT_DELAY_MS: i32 = 500;
const MAX_RECONNECT_DELAY_MS: i32 = 10_000;

thread_local! {
    static RECONNECT_DELAY_MS: Cell<i32> = const { Cell::new(INITIAL_RECONNECT_DELAY_MS) };
}

fn query_position(square: &str) -> Option<Element> {
    document()
        .query_selector(&format!("[data-square=\"{}\"]", square))
        .unwrap()
}

fn handle_message(chess_board_signals: ChessBoardSignals, msg: ServerMsg) {
    match msg {
        ServerMsg::Move { piece, from, to } => {
            let last_move = (
                Position::parse_square(&from).ok().flatten(),
                Position::parse_square(&to).ok().flatten(),
            );
            chess_board_signals.last_move().set(Some(last_move));

            let update_board = {
                let (from, to) = (from.clone(), to.clone());
                move || chess_board_signals.move_piece(piece.clone(), from.clone(), to.clone())
            };

            if from != DELETED && to != DELETED {
                if let Some(element) = query_position(&from) {
                    element.class_list_remove(&format!("square-{}", from));
                    element.class_list_add(&format!("square-{}", to));

                    let window = web_sys::window().unwrap();
                    window.set_timeout_callback(update_board, 100);
                }
            } else {
                update_board();
            }
        }
        ServerMsg::SyncBoard { room, fen, trash } => {
            let white_view = chess_board_signals
                .chess_board()
                .with_untracked(|cb| cb.white_view());
            let Ok(new_chessboard) = ChessBoardBuilder::new()
                .fen(&fen)
                .deleted_stones(&trash)
                .is_white_view(white_view)
                .validation(false)
                .build()
            else {
                log::error!("Invalid board: {}|{}", fen, trash);
                return;
            };

            chess_board_signals.should_render().set(false);
            chess_board_signals.selected().set(None);
            chess_board_signals.last_move().set(None);

            chess_board_signals
                .stones_signals()
                .update(|stones_signals| {
                    stones_signals.clear_board_stones();
                    stones_signals.clear_deleted_stones();
                });

            chess_board_signals.room_status().update(|room_status| {
                if let Some(room_status) = room_status {
                    room_status.set_name(&room);
                    room_status.set_checkmate(false);
                } else {
                    *room_status = Some(RoomStatus::new(&room));
                }
            });

            chess_board_signals.chess_board().set(new_chessboard);

            let positions_and_stones = chess_board_signals
                .chess_board()
                .with_untracked(|cb| cb.cloned_stones_and_positions());
            let deleted_stones = chess_board_signals
                .chess_board()
                .with_untracked(|cb| cb.cloned_deleted_stones());

            chess_board_signals
                .stones_signals()
                .update(|stones_signals| {
                    for (position, stone) in positions_and_stones {
                        stones_signals.add_board_stone(position, stone);
                    }
                    for stone in deleted_stones {
                        stones_signals.add_deleted_stone(stone);
                    }
                });

            chess_board_signals.should_render().set(true);
        }
        ServerMsg::SyncUsers { room, users } => {
            let room_status = chess_board_signals.room_status().get_untracked();

            if let Some(mut room_status) = room_status {
                for user in users.iter().cloned().map(User::from) {
                    if let Some(old_user) = room_status.get_user(&user.id()) {
                        old_user.set(user);
                    }
                }

                room_status.sync_users(users);

                chess_board_signals.room_status().set(Some(room_status));
            } else {
                let mut new_room_status = RoomStatus::new(&room);
                new_room_status.sync_users(users);
                chess_board_signals.room_status().set(Some(new_room_status));
            }
        }
        ServerMsg::SyncOptions { validation, sync } => {
            chess_board_signals.room_status().update(|room_status| {
                if let Some(room_status) = room_status {
                    room_status.set_options(validation, sync);
                }
            });
        }
        ServerMsg::UserJoined { user } => {
            let room_status = chess_board_signals.room_status().get_untracked();
            let new_user = User::from(user);

            if let Some(mut room_status) = room_status {
                if let Some(old_user) = room_status.get_user(&new_user.id()) {
                    old_user.update(|user| {
                        user.set_username(&new_user.username());
                        user.connect();
                    });
                } else {
                    room_status.add_user(new_user);
                    chess_board_signals.room_status().set(Some(room_status));
                }
            }
        }
        ServerMsg::UserLeft { id } => {
            chess_board_signals.room_status().update(|room_status| {
                if let Some(room_status) = room_status {
                    room_status.remove_user(&id);
                }
            });
        }
        ServerMsg::UserPresence { id, presence } => {
            let user = chess_board_signals
                .room_status()
                .get_untracked()
                .and_then(|room_status| room_status.get_user(&id));

            if let Some(user) = user {
                user.update(|u| match presence {
                    Presence::Online => u.connect(),
                    Presence::Away => u.disconnect(),
                });
            }
        }
        ServerMsg::Notify { level, message } => chess_board_signals.notify(level, &message),
        ServerMsg::Checkmate => {
            chess_board_signals.room_status().update(|room_status| {
                if let Some(room_status) = room_status {
                    room_status.set_checkmate(true);
                }
            });
        }
        ServerMsg::Chat { name, text } => log::info!("{}: {}", name, text),
        ServerMsg::Game { game } => {
            chess_board_signals.clock_synced_at().set(now_ms());
            chess_board_signals.game().set(game);
        }
        ServerMsg::GameCreated { room } => chess_board_signals.created_game().set(Some(room)),
        ServerMsg::RoomGone { room } => {
            log::info!("Room {} is gone", room);
            chess_board_signals.notify(NotifyType::Warning, "That game has ended and is no longer available");
            chess_board_signals.redirect().set(Some(MAIN_ROOM.to_string()));
        }
        ServerMsg::Replaced => {
            chess_board_signals.connection().set(Connection::Replaced);
            chess_board_signals.notify(NotifyType::Warning, "The board was opened in another tab");
        }
        ServerMsg::Error { message } => {
            log::error!("Server error: {}", message);
            chess_board_signals.notify(NotifyType::Error, &message);
        }
    }
}

fn on_message_callback(chess_board_signals: ChessBoardSignals) -> Closure<dyn FnMut(MessageEvent)> {
    Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
        let Some(text) = e.data().as_string() else {
            return;
        };

        match serde_json::from_str::<ServerMsg>(&text) {
            Ok(msg) => handle_message(chess_board_signals, msg),
            Err(err) => log::error!("Invalid server message {:?}: {}", text, err),
        }
    })
}

/// Opens the socket again after a growing delay
fn schedule_reconnect(chess_board_signals: ChessBoardSignals) {
    let delay = RECONNECT_DELAY_MS.get();
    RECONNECT_DELAY_MS.set((delay * 2).min(MAX_RECONNECT_DELAY_MS));

    let window = web_sys::window().unwrap();
    window.set_timeout_callback(move || chess_board_signals.start_websocket(), delay);
}

/// The socket closed before opening: either the server is unreachable, or it
/// refused a session cookie that doesn't verify, which needs a new username
fn check_session(chess_board_signals: ChessBoardSignals) {
    let window = web_sys::window().unwrap();
    let on_response = Closure::<dyn FnMut(_)>::new(move |response: JsValue| {
        let unauthorized = response
            .dyn_into::<Response>()
            .is_ok_and(|response| response.status() == 401);

        if unauthorized {
            expire_cookie("session_token");
            chess_board_signals.needs_username().set(true);
        } else {
            schedule_reconnect(chess_board_signals);
        }
    });
    let on_error = Closure::<dyn FnMut(_)>::new(move |_: JsValue| {
        schedule_reconnect(chess_board_signals);
    });

    let _ = window
        .fetch_with_str("/sessions")
        .then2(&on_response, &on_error);
    on_response.forget();
    on_error.forget();
}

pub fn start_websocket(chess_board_signals: ChessBoardSignals) -> Result<WebSocket, JsValue> {
    let location = web_sys::window().unwrap().location();

    let proto = location
        .protocol()
        .unwrap()
        .starts_with("https")
        .then(|| "wss")
        .unwrap_or("ws");
    let ws_uri = format!(
        "{proto}://{host}/ws",
        proto = proto,
        host = location.host().unwrap()
    );

    let ws = WebSocket::new(&ws_uri)?;
    chess_board_signals.connection().set(Connection::Connecting);

    let onmessage_callback = on_message_callback(chess_board_signals);
    // set message event handler on WebSocket
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    // forget the callback to keep it alive
    onmessage_callback.forget();

    let onerror_callback = Closure::<dyn FnMut(_)>::new(move |e: ErrorEvent| {
        log::error!("Error event: {:?}", e);
    });
    ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
    onerror_callback.forget();

    // The room is joined by an effect watching the connection and the URL
    let onopen_callback = Closure::<dyn FnMut()>::new(move || {
        RECONNECT_DELAY_MS.set(INITIAL_RECONNECT_DELAY_MS);
        chess_board_signals.connection().set(Connection::Open);
    });
    ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();

    let onclose_callback = Closure::<dyn FnMut(_)>::new(move |_: CloseEvent| {
        let previous = chess_board_signals.connection().get_untracked();
        chess_board_signals.socket().set(None);

        let room_status = chess_board_signals.room_status().get_untracked();
        if let Some(room_status) = room_status {
            for user in room_status.users() {
                user.update(|u| u.logout());
            }

            chess_board_signals.room_status().set(Some(room_status));
        }

        match previous {
            // another tab took over, reconnecting would take it back
            Connection::Replaced => {}
            Connection::Open => {
                chess_board_signals.connection().set(Connection::Closed);
                schedule_reconnect(chess_board_signals);
            }
            Connection::Connecting | Connection::Closed => {
                chess_board_signals.connection().set(Connection::Closed);
                check_session(chess_board_signals);
            }
        }
    });
    ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
    onclose_callback.forget();

    Ok(ws)
}
