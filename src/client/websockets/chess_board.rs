use leptos::{SignalGetUntracked, SignalSet, SignalUpdate};
use wasm_bindgen::prelude::*;
use web_sys::{CloseEvent, Element, ErrorEvent, MessageEvent, WebSocket};

use crate::{
    entities::{
        chess_board::{ChessBoard, ChessBoardSignals},
        room::{RoomStatus, User, UserStatus},
    },
    utils::{
        class_list::ClassListExt,
        elements::{self, document},
    },
};

fn query_position(square: &str) -> Option<Element> {
    document()
        .query_selector(&format!("[data-square=\"{}\"]", square))
        .unwrap()
}

fn on_message_callback(chess_board_signals: ChessBoardSignals) -> Closure<dyn FnMut(MessageEvent)> {
    Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
        // Handle difference Text/Binary,...
        if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
            log::debug!("message event, received Text: {:?}", txt);

            let text: String = txt.as_string().unwrap();
            if text.starts_with('/') {
                let (cmd, input) = text.split_once(" ").unwrap_or((text.as_str(), ""));

                match cmd {
                    "/move" => {
                        let mut input = input.split(" ");
                        let piece_data = input.next().unwrap();

                        let old_pos = input.next().unwrap();
                        let new_pos = input.next().unwrap();

                        if old_pos == new_pos {
                            return;
                        }

                        let query_old = query_position(old_pos);
                        let query_new = query_position(new_pos);
                        if old_pos == "deleted" {
                            if let Some(element) = elements::query_selector(&format!(
                                ".deleted[data-piece=\"{}\"]",
                                piece_data
                            )) {
                                elements::restore_piece(&element);
                                element.set_attribute("data-square", &new_pos).unwrap();
                                element.class_list_add(&format!("square-{}", new_pos));

                                if let Some(element) = query_new {
                                    elements::soft_delete_piece(&element);
                                    element.set_attribute("data-square", "deleted").unwrap();
                                }
                            };
                        } else if new_pos == "deleted" {
                            if let Some(element) = query_old {
                                elements::soft_delete_piece(&element);
                                element.set_attribute("data-square", "deleted").unwrap();
                            }
                        } else {
                            if let Some(element) = query_old {
                                element.set_attribute("data-square", &new_pos).unwrap();
                                element.class_list_remove(&format!("square-{}", old_pos));
                                element.class_list_add(&format!("square-{}", new_pos));

                                if let Some(element) = query_new {
                                    elements::soft_delete_piece(&element);
                                    element.set_attribute("data-square", "deleted").unwrap();
                                }
                            }
                        }
                    }
                    "/sync_board" => {
                        chess_board_signals.should_render().set(false);
                        let mut input = input.split("|");

                        let room_name = input.next().unwrap();
                        let fen = input.next().unwrap();
                        let trash = input.next().unwrap();

                        chess_board_signals.room_status().update(|room_status| {
                            if let Some(room_status) = room_status {
                                room_status.set_name(room_name);
                            } else {
                                *room_status =
                                    Some(RoomStatus::new(chess_board_signals.cx(), room_name));
                            }
                        });

                        chess_board_signals.chess_board().update(|chessboard| {
                            let reset_count = chessboard.reset_count() + 1;

                            let mut new_chessboard = ChessBoard::new(fen);
                            if chessboard.white_view() != new_chessboard.white_view() {
                                new_chessboard.flip();
                            }
                            new_chessboard.set_trash_from_str(trash);
                            new_chessboard.set_reset_count(reset_count);
                            *chessboard = new_chessboard;
                        });
                        chess_board_signals.should_render().set(true);
                    }
                    "/sync_users" => {
                        let mut input = input.split("|");
                        let room_name = input.next().unwrap();
                        let users: Vec<String> =
                            input.next().unwrap().split(",").map(String::from).collect();

                        let room_status = chess_board_signals.room_status().get_untracked();

                        if let Some(mut room_status) = room_status {
                            for user in users.iter().map(|user| user.parse::<User>().unwrap()) {
                                if let Some(old_user) = room_status.get_user(&user.id()) {
                                    match user.status() {
                                        UserStatus::Away => old_user.update(|u| u.disconnect()),
                                        UserStatus::Online => old_user.update(|u| u.connect()),
                                        _ => {}
                                    }
                                }
                            }

                            room_status.sync_users(users);

                            chess_board_signals.room_status().set(Some(room_status));
                        } else {
                            let mut new_room_status =
                                RoomStatus::new(chess_board_signals.cx(), room_name);
                            new_room_status.sync_users(users);
                            chess_board_signals.room_status().set(Some(new_room_status));
                        }
                    }
                    "/add_user" => {
                        let room_status = chess_board_signals.room_status().get_untracked();

                        let new_user = input.parse::<User>().unwrap();

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
                    "/remove_user" => {
                        chess_board_signals.room_status().update(|room_status| {
                            if let Some(room_status) = room_status {
                                room_status.remove_user(input);
                            }
                        });
                    }
                    "/disconnect_user" => {
                        let (id, _) = input.split_once(":").unwrap_or((input, ""));

                        let user = chess_board_signals
                            .room_status()
                            .get_untracked()
                            .and_then(|room_status| room_status.get_user(id));

                        if let Some(user) = user {
                            user.update(|u| u.disconnect());
                        }
                    }
                    "/connect_user" => {
                        let (id, _) = input.split_once(":").unwrap_or((input, ""));

                        let user = chess_board_signals
                            .room_status()
                            .get_untracked()
                            .and_then(|room_status| room_status.get_user(id));

                        if let Some(user) = user {
                            user.update(|u| u.connect());
                        }
                    }
                    _ => {}
                }
            }
        } else {
            log::debug!("message event, received Unknown: {:?}", e.data());
        }
    })
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

    // Connect to an echo server
    let ws = WebSocket::new(&ws_uri)?;

    let onmessage_callback = on_message_callback(chess_board_signals);
    // set message event handler on WebSocket
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    // forget the callback to keep it alive
    onmessage_callback.forget();

    let onerror_callback = Closure::<dyn FnMut(_)>::new(move |e: ErrorEvent| {
        log::debug!("error event: {:?}", e);
    });
    ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
    onerror_callback.forget();

    let onopen_callback = Closure::<dyn FnMut()>::new(move || {
        log::debug!("socket opened");
    });
    ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();

    let onclose_callback = Closure::<dyn FnMut(_)>::new(move |e: CloseEvent| {
        log::debug!("socket closed: {:?}", e);
        chess_board_signals.socket().set(None);
        let room_status = chess_board_signals.room_status().get_untracked();
        if let Some(room_status) = room_status {
            for user in room_status.users() {
                user.update(|u| u.logout());
            }

            chess_board_signals.room_status().set(Some(room_status));
        }
    });
    ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
    onclose_callback.forget();

    Ok(ws)
}
