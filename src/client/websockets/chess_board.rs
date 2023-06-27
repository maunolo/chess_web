use leptos::{SignalSet, SignalUpdate, WriteSignal};
use wasm_bindgen::prelude::*;
use web_sys::{Element, ErrorEvent, MessageEvent, WebSocket};

use crate::{
    entities::chess_board::ChessBoard,
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

fn on_message_callback(
    chessboard: WriteSignal<ChessBoard>,
    should_render: WriteSignal<bool>,
) -> Closure<dyn FnMut(MessageEvent)> {
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
                        should_render.set(false);
                        chessboard.update(|chessboard| {
                            let mut input = input.split("|");

                            let fen = input.next().unwrap();
                            let trash = input.next().unwrap();
                            let reset_count = chessboard.reset_count() + 1;

                            let mut new_chessboard = ChessBoard::new(fen);
                            if chessboard.white_view() != new_chessboard.white_view() {
                                new_chessboard.flip();
                            }
                            new_chessboard.set_trash_from_str(trash);
                            new_chessboard.set_reset_count(reset_count);
                            *chessboard = new_chessboard;
                        });
                        should_render.set(true);
                    }
                    _ => {}
                }
            }
        } else {
            log::debug!("message event, received Unknown: {:?}", e.data());
        }
    })
}

pub fn start_websocket(
    chessboard: WriteSignal<ChessBoard>,
    should_render: WriteSignal<bool>,
    username: String,
) -> Result<WebSocket, JsValue> {
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
    let onmessage_callback = on_message_callback(chessboard, should_render);
    // set message event handler on WebSocket
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    // forget the callback to keep it alive
    onmessage_callback.forget();

    let onerror_callback = Closure::<dyn FnMut(_)>::new(move |e: ErrorEvent| {
        log::debug!("error event: {:?}", e);
    });
    ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
    onerror_callback.forget();

    let clone_ws = ws.clone();
    let onopen_callback = Closure::<dyn FnMut()>::new(move || {
        log::debug!("socket opened");

        let msg = format!("/username {}", username);
        match clone_ws.send_with_str(&msg) {
            Ok(_) => log::debug!("message sent: {}", msg),
            Err(err) => log::error!("error sending message: {:?}", err),
        }
    });
    ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();

    Ok(ws)
}
