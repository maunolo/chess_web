use leptos::{SignalGetUntracked, SignalSet, SignalUpdate, SignalWithUntracked};
use wasm_bindgen::prelude::*;
use web_sys::{CloseEvent, Element, ErrorEvent, MessageEvent, WebSocket};

use crate::{
    entities::{
        chess_board::{signals::ChessBoardSignals, ChessBoardBuilder},
        notification::NotifyType,
        room::{RoomStatus, User, UserStatus},
    },
    utils::{class_list::ClassListExt, elements::document, WindowExt},
};

fn query_position(square: &str) -> Option<Element> {
    document()
        .query_selector(&format!("[data-square=\"{}\"]", square))
        .unwrap()
}

fn on_message_callback(chess_board_signals: ChessBoardSignals) -> Closure<dyn FnMut(MessageEvent)> {
    Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
        if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
            let text: String = txt.as_string().unwrap();
            if text.starts_with('/') {
                let (cmd, input) = text.split_once(" ").unwrap_or((text.as_str(), ""));

                match cmd {
                    "/move" => {
                        let mut input = input.split(" ");
                        let piece_data = input.next().unwrap().to_string();

                        let old_pos = input.next().unwrap().to_string();
                        let new_pos = input.next().unwrap().to_string();

                        let old_pos_clone = old_pos.clone();
                        let new_pos_clone = new_pos.clone();
                        let update_board = move || {
                            chess_board_signals.move_piece(
                                piece_data.to_string(),
                                old_pos.to_string(),
                                new_pos.to_string(),
                            )
                        };

                        if old_pos_clone != "deleted" && new_pos_clone != "deleted" {
                            if let Some(piece) = query_position(&old_pos_clone) {
                                piece.class_list_remove(&format!("square-{}", old_pos_clone));
                                piece.class_list_add(&format!("square-{}", new_pos_clone));

                                let window = web_sys::window().unwrap();
                                window.set_timeout_callback(update_board, 100);
                            }
                        } else {
                            update_board();
                        };
                    }
                    "/sync_board" => {
                        chess_board_signals.should_render().set(false);
                        let mut input = input.split("|");

                        let room_name = input.next().unwrap();
                        let fen = input.next().unwrap();
                        let trash = input.next().unwrap();

                        chess_board_signals
                            .stones_signals()
                            .update(|stones_signals| {
                                stones_signals.clear_board_stones();
                                stones_signals.clear_deleted_stones();
                            });

                        chess_board_signals.room_status().update(|room_status| {
                            if let Some(room_status) = room_status {
                                room_status.set_name(room_name);
                            } else {
                                *room_status =
                                    Some(RoomStatus::new(chess_board_signals.cx(), room_name));
                            }
                        });

                        chess_board_signals.chess_board().update(|chessboard| {
                            let new_chessboard = ChessBoardBuilder::new()
                                .fen(fen)
                                .deleted_stones(trash)
                                .is_white_view(chessboard.white_view())
                                .validation(false)
                                .build()
                                .unwrap();

                            *chessboard = new_chessboard;
                        });

                        let positions_and_stones = chess_board_signals
                            .chess_board()
                            .with_untracked(|cb| cb.cloned_stones_and_positions());
                        let deleted_stones = chess_board_signals
                            .chess_board()
                            .with_untracked(|cb| cb.cloned_deleted_stones());
                        let cx = chess_board_signals.cx();

                        chess_board_signals
                            .stones_signals()
                            .update(|stones_signals| {
                                for (position, stone) in positions_and_stones {
                                    stones_signals.add_board_stone(cx, position, stone);
                                }
                                for stone in deleted_stones {
                                    stones_signals.add_deleted_stone(cx, stone);
                                }
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
                    "/sync_options" => {
                        chess_board_signals.room_status().update(|room_status| {
                            if let Some(room_status) = room_status {
                                room_status.set_options_from_str(input);
                            }
                        });
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
                    "/notify" => {
                        let (notify_type, msg) = input.split_once(" ").unwrap_or((input, ""));
                        let notify_type = match notify_type {
                            "error" => NotifyType::Error,
                            "warning" => NotifyType::Warning,
                            "success" => NotifyType::Success,
                            _ => return,
                        };

                        chess_board_signals.notification().update(|notification| {
                            notification.notify_type = notify_type;
                            notification.message = msg.to_string();
                            notification.disable();
                        });

                        chess_board_signals.notification().update(|notification| {
                            notification.enable();
                        });
                    }
                    _ => {}
                }
            }
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
        log::error!("Error event: {:?}", e);
    });
    ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
    onerror_callback.forget();

    // let onopen_callback = Closure::<dyn FnMut()>::new(move || {
    //     log::info!("socket opened");
    // });
    // ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    // onopen_callback.forget();

    let onclose_callback = Closure::<dyn FnMut(_)>::new(move |_: CloseEvent| {
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
