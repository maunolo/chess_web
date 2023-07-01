use leptos::*;

use crate::{
    components::forms::{join::Join, username::Username},
    entities::chess_board::{ChessBoard, ChessBoardSignals},
};

use cfg_if::cfg_if;

#[allow(unused_variables)]
fn toggle_sub_menu(sub_menu_timeout_id: RwSignal<Option<i32>>, show_menu: bool) {
    cfg_if! {
        if #[cfg(not(feature = "ssr"))] {
            use crate::utils::WindowExt;

            let window = web_sys::window().unwrap();
            let sub_menu = window
                .document()
                .unwrap()
                .get_element_by_id("sub-menu")
                .unwrap();

            if show_menu {
                sub_menu.set_class_name("sub-menu sub-menu--is-active")
            } else {
                if let Some(id) = window.set_timeout_callback(
                    move || sub_menu.set_class_name("sub-menu"),
                    250,
                ) {
                    sub_menu_timeout_id.set(Some(id))
                };
            }
        }
    }
}

fn clear_timeout(id: Option<i32>) {
    if let Some(id) = id {
        let window = web_sys::window().unwrap();

        window.clear_timeout_with_handle(id);
    }
}

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
pub fn Overlay(
    cx: Scope,
    chess_board: WriteSignal<ChessBoard>,

    chess_board_socket: RwSignal<Option<web_sys::WebSocket>>,
    chess_board_signals: ChessBoardSignals,
) -> impl IntoView {
    let (show_form, set_show_form) = create_signal(cx, Form::None);
    let (show_menu, set_show_menu) = create_signal(cx, false);
    let (room_status, set_room_status) = create_signal::<Option<RoomStatus>>(cx, None);
    let sub_menu_timeout_id = create_rw_signal::<Option<i32>>(cx, None);
    let mut chess_board_signals = chess_board_signals;

    chess_board_signals.set_room_status(set_room_status);

    create_effect(cx, move |_| {
        use crate::utils::WindowExt;

        if let Some(username) = get_local_username() {
            chess_board_socket.set(chess_board_signals.start_websocket(username));
        } else {
            let window = web_sys::window().unwrap();
            window.set_timeout_callback(move || set_show_form.set(Form::Username), 0);
        }

        toggle_sub_menu(sub_menu_timeout_id, false);
    });

    let join = move |_| {
        set_show_form.set(Form::Join);
    };

    let reset = move |_| {
        if let Some(socket) = chess_board_socket.get().as_ref() {
            match socket.send_with_str("/reset") {
                Ok(_) => log::debug!("message successfully sent: {:?}", "/reset"),
                Err(err) => log::debug!("error sending message: {:?}", err),
            }
        }
    };

    let toggle_menu = move |_| {
        set_show_menu.update(|show_menu| {
            clear_timeout(sub_menu_timeout_id.get());
            toggle_sub_menu(sub_menu_timeout_id, !*show_menu);

            *show_menu = !*show_menu;
        });
    };

    let join_submit = move |e: web_sys::SubmitEvent| {
        e.prevent_default();
        if let Some(socket) = chess_board_socket.get().as_ref() {
            let target = e.target().unwrap();
            let form = crate::utils::js_cast::<web_sys::HtmlFormElement, _>(target);

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
        let form = crate::utils::js_cast::<web_sys::HtmlFormElement, _>(target);

        if let Some(form) = form {
            let data = web_sys::FormData::new_with_form(&form).unwrap();
            let username = data.get("username").as_string().unwrap();
            set_local_username(username.clone());
            chess_board_socket.set(chess_board_signals.start_websocket(username));
        }
        set_show_form.set(Form::None);
    };

    let menu_css = move || {
        if show_menu.get() {
            "menu menu--is-active"
        } else {
            "menu"
        }
    };

    let menu_btn_css = move || {
        if show_menu.get() {
            "menu-btn menu-btn--is-active"
        } else {
            "menu-btn"
        }
    };

    view! { cx,
        <>
            <div class=menu_css>
                <div class="menu-header">
                    <button
                        class=menu_btn_css
                        on:click=toggle_menu
                    >
                        <span class="line"></span>
                    </button>
                    <h1 class="room-title">{
                        move || room_status.with(|status| status.as_ref().map(|s| s.name.clone()).unwrap_or("Chess".to_owned()))
                    }</h1>
                </div>
                <div class="sub-menu sub-menu--is-active" id="sub-menu">
                    <button
                        class="sub-menu-item"
                        on:click=move |_| { chess_board.update(|cb| cb.flip()) }
                    >
                        "Flip"
                    </button>
                    <button
                        class="sub-menu-item"
                        on:click=reset
                    >
                        "Reset"
                    </button>
                    <button
                        class="sub-menu-item"
                        on:click=join
                    >
                        "Join"
                    </button>
                </div>
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
