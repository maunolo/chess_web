use wasm_bindgen::prelude::*;
use web_sys::{Element, ErrorEvent, MessageEvent, WebSocket};

use crate::utils::{
    class_list::ClassListExt,
    elements::{self, document},
};

fn query_position(square: &str) -> Option<Element> {
    document()
        .query_selector(&format!("[data-square=\"{}\"]", square))
        .unwrap()
}

fn on_message_callback() -> Closure<dyn FnMut(MessageEvent)> {
    Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
        // Handle difference Text/Binary,...
        if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
            log::debug!("message event, received Text: {:?}", txt);

            let text: String = txt.as_string().unwrap();
            let mut input = text.split(" ");

            let cmd = input.next().unwrap();

            match cmd {
                "move" => {
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
                _ => {}
            }
        } else {
            log::debug!("message event, received Unknown: {:?}", e.data());
        }
    })
}

pub fn start_websocket() -> Result<WebSocket, JsValue> {
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
    let onmessage_callback = on_message_callback();
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

    Ok(ws)
}
