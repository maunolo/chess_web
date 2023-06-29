use leptos::*;

use crate::{
    components::forms::{join::Join, username::Username},
    entities::chess_board::{ChessBoard, ChessBoardSignals},
};

use cfg_if::cfg_if;

#[derive(Clone)]
#[allow(dead_code)]
pub enum Form {
    None,
    Join,
    Username,
}

#[allow(dead_code)]
pub struct RoomStatus {
    name: String,
    users: Vec<String>,
}

#[allow(dead_code)]
impl RoomStatus {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            users: vec![],
        }
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

fn set_local_username(username: String) {
    let local_storage = web_sys::window().map(|w| w.local_storage());
    match local_storage {
        Some(Ok(Some(local_storage))) => {
            let _ = local_storage.set_item("username", &username);
        }
        _ => {
            log::debug!("No local storage available to set");
        }
    }
}

#[allow(dead_code)]
fn get_local_username() -> Option<String> {
    let local_storage = web_sys::window()?.local_storage();
    match local_storage {
        Ok(Some(local_storage)) => local_storage.get_item("username").unwrap(),
        _ => {
            log::debug!("No local storage available to get");
            None
        }
    }
}

#[component]
#[allow(unused_variables)]
pub fn Overlay<R>(
    cx: Scope,
    chess_board: WriteSignal<ChessBoard>,
    reset: R,
    chess_board_socket: RwSignal<Option<web_sys::WebSocket>>,
    chess_board_signals: ChessBoardSignals,
) -> impl IntoView
where
    R: Fn(web_sys::MouseEvent) -> () + 'static,
{
    let (show_form, set_show_form) = create_signal(cx, Form::None);
    let (room_status, set_room_status) = create_signal::<Option<RoomStatus>>(cx, None);
    let mut chess_board_signals = chess_board_signals;

    chess_board_signals.set_room_status(set_room_status);

    create_effect(cx, move |_| {
        cfg_if! {
            if #[cfg(not(feature = "ssr"))] {
                use wasm_bindgen::JsCast;

                if let Some(username) = get_local_username() {
                    chess_board_socket.set(chess_board_signals.start_websocket(username));
                } else {
                    let window = web_sys::window().unwrap();
                    let open_username_form = wasm_bindgen::prelude::Closure::<dyn FnMut()>::new(move || {
                        set_show_form.set(Form::Username);
                    });
                    let _ = window.set_timeout_with_callback(open_username_form.as_ref().unchecked_ref());
                    open_username_form.forget();
                }

                let username = Some(web_sys::window().unwrap().local_storage());
            } else {
                let username: Option<String> = None;
            }
        }
    });

    let join = move |_| {
        set_show_form.set(Form::Join);
    };

    let join_submit = move |e: web_sys::SubmitEvent| {
        e.prevent_default();
        if let Some(socket) = chess_board_socket.get().as_ref() {
            let target = e.target().unwrap();
            cfg_if! {
                if #[cfg(feature = "ssr")] { let form : Option < web_sys::HtmlFormElement
                    > = None; } else { let form = Some(wasm_bindgen::JsCast::dyn_into::<
                                                       web_sys::HtmlFormElement > (target).unwrap()); }
            }
            if let Some(form) = form {
                let data = web_sys::FormData::new_with_form(&form).unwrap();
                let room = data.get("room").as_string().unwrap();
                let msg = format!("/join {}", room);
                match socket.send_with_str(&msg) {
                    Ok(_) => log::debug!("message successfully sent: {:?}", msg),
                    Err(err) => log::debug!("error sending message: {:?}", err),
                }
            }
            set_show_form.set(Form::None);
        }
    };

    let username_submit = move |e: web_sys::SubmitEvent| {
        e.prevent_default();
        let target = e.target().unwrap();
        cfg_if! {
            if #[cfg(feature = "ssr")] { let form : Option < web_sys::HtmlFormElement > =
                None; } else { let form = Some(wasm_bindgen::JsCast::dyn_into::<
                                               web_sys::HtmlFormElement > (target).unwrap()); }
        }
        if let Some(form) = form {
            let data = web_sys::FormData::new_with_form(&form).unwrap();
            let username = data.get("username").as_string().unwrap();
            set_local_username(username.clone());
            chess_board_socket.set(chess_board_signals.start_websocket(username));
        }
        set_show_form.set(Form::None);
    };

    view! { cx,
        <>
            <div class="pointer-events-none flex flex-none fixed bottom-0 z-30 sm:bottom-auto sm:right-auto sm:left-0 sm:top-0">
                <h1 class="text-xl bg-neutral-300 font-extrabold px-2 py-2 rounded-t-lg sm:rounded-t-none sm:rounded-br-lg">{
                    move || room_status.with(|status| status.as_ref().map(|s| s.name.clone()).unwrap_or("Chess".to_owned()))
                }</h1>
            </div>
            <div class="pointer-events-none flex flex-none fixed gap-2 top-0 z-30 py-2 px-2 justify-center sm:w-24 sm:left-0 sm:flex-col sm:top-auto">
                <button
                    class="pointer-events-auto h-fit w-20 py-2 rounded bg-neutral-300 hover:bg-neutral-400 sm:h-auto"
                    on:click=move |_| { chess_board.update(|cb| cb.flip()) }
                >
                    "Flip"
                </button>
                <button
                    class="pointer-events-auto h-fit w-20 py-2 rounded bg-neutral-300 hover:bg-neutral-400 sm:h-auto"
                    on:click=reset
                >
                    "Reset"
                </button>
                <button
                    class="pointer-events-auto h-fit w-20 py-2 rounded bg-neutral-300 hover:bg-neutral-400 sm:h-auto"
                    on:click=join
                >
                    "Join"
                </button>
            </div>
            {move || {
                if !matches!(show_form.get(), Form::None) {
                    view! { cx,
                        <div class="z-40 flex absolute w-full h-full justify-center items-center bg-neutral-900/30">
                            {move || {
                                match show_form.get() {
                                    Form::Join => {
                                        view! { cx,
                                            <>
                                                <Join submit=join_submit/>
                                            </>
                                        }
                                    }
                                    Form::Username => {
                                        view! { cx,
                                            <>
                                                <Username submit=username_submit/>
                                            </>
                                        }
                                    }
                                    Form::None => {
                                        view! { cx,
                                            <>
                                                <div class="hidden"></div>
                                            </>
                                        }
                                    }
                                }
                            }}
                        </div>
                    }
                } else {
                    view! { cx, <div class="hidden"></div> }
                }
            }}
        </>
    }
}
