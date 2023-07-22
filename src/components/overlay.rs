use leptos::*;

use crate::{
    components::{
        forms::{join::Join, options::Options, username::Username},
        menu::Menu,
        status_menu::StatusMenu,
    },
    entities::chess_board::signals::ChessBoardSignals,
    utils::{get_cookie_value, jwt::decode, SessionPayload, WindowExt},
};

#[derive(Clone)]
#[allow(dead_code)]
pub enum Form {
    None,
    Join,
    Username,
    Options,
}

pub fn toggle_sub_menu(
    menu_timeout_id: RwSignal<Option<i32>>,
    show_menu: bool,
    menu_base_class: &str,
) {
    use crate::utils::class_list::ClassListExt;

    let window = web_sys::window().unwrap();
    let sub_menu = window
        .document()
        .unwrap()
        .get_element_by_id(menu_base_class)
        .unwrap();

    let is_active_class = format!("{}--is-active", menu_base_class);

    if show_menu {
        sub_menu.class_list_add(&is_active_class)
    } else {
        if let Some(id) =
            window.set_timeout_callback(move || sub_menu.class_list_remove(&is_active_class), 250)
        {
            menu_timeout_id.set(Some(id))
        };
    }
}

pub fn clear_timeout(id: Option<i32>) {
    if let Some(id) = id {
        let window = web_sys::window().unwrap();

        window.clear_timeout_with_handle(id);
    }
}

fn set_username(username: &str, chess_board_signals: ChessBoardSignals) {
    let window = web_sys::window().unwrap();
    window.post_user_name(username, chess_board_signals);
}

pub fn get_user_payload() -> Option<SessionPayload> {
    if let Some(session_token) = get_cookie_value("session_token") {
        let Ok(token) = decode::<SessionPayload>(&session_token) else {
            log::error!("Session token is not a valid JWT: {}", session_token);
            return None;
        };
        Some(token.claims().clone())
    } else {
        None
    }
}

#[component]
pub fn Overlay(cx: Scope, chess_board_signals: ChessBoardSignals) -> impl IntoView {
    let show_form = create_rw_signal(cx, Form::None);

    create_effect(cx, move |_| {
        use crate::utils::WindowExt;

        if let Some(_) = get_user_payload().map(|payload| payload.name) {
            chess_board_signals.start_websocket();
        } else {
            let window = web_sys::window().unwrap();
            window.set_timeout_callback(move || show_form.set(Form::Username), 0);
        };
    });

    let join_submit = move |e: web_sys::SubmitEvent| {
        e.prevent_default();
        if let Some(socket) = chess_board_signals.socket().get().as_ref() {
            let target = e.target().unwrap();
            let form = crate::utils::js_cast::<web_sys::HtmlFormElement, _>(target);

            if let Some(form) = form {
                let data = web_sys::FormData::new_with_form(&form).unwrap();
                let room = data.get("room").as_string().unwrap();
                let msg = format!("/join {}", room);
                match socket.send_with_str(&msg) {
                    Ok(_) => {}
                    Err(err) => log::error!("error sending message: {:?}", err),
                }
            }
            show_form.set(Form::None);
        }
    };

    let username_submit = move |e: web_sys::SubmitEvent| {
        e.prevent_default();
        let target = e.target().unwrap();
        let form = crate::utils::js_cast::<web_sys::HtmlFormElement, _>(target);

        if let Some(form) = form {
            let data = web_sys::FormData::new_with_form(&form).unwrap();
            let username = data.get("username").as_string().unwrap();
            set_username(&username, chess_board_signals);
        }
        show_form.set(Form::None);
    };

    let options_submit = move |e: web_sys::SubmitEvent| {
        e.prevent_default();
        let target = e.target().unwrap();
        let form = crate::utils::js_cast::<web_sys::HtmlFormElement, _>(target);

        if let Some(form) = form {
            let data = web_sys::FormData::new_with_form(&form).unwrap();
            let validation = data
                .get("validation")
                .as_string()
                .map(|s| s == "on")
                .unwrap_or(false);
            let sync = data
                .get("sync")
                .as_string()
                .map(|s| s == "on")
                .unwrap_or(false);
            log::debug!("validation: {:?}, sync: {:?}", validation, sync);
            chess_board_signals.room_status().update(|status| {
                if let Some(status) = status.as_mut() {
                    if validation {
                        status.enable_validation();
                    } else {
                        status.disable_validation();
                    }

                    if sync {
                        status.enable_sync();
                    } else {
                        status.disable_sync();
                    }
                };
            });

            let options_msg = chess_board_signals.room_status().with_untracked(|rs| {
                rs.as_ref()
                    .map(|rs| rs.options_string())
                    .unwrap_or_default()
            });

            if let Some(socket) = chess_board_signals.socket().get().as_ref() {
                match socket.send_with_str(&format!("/options {}", options_msg)) {
                    Ok(_) => {}
                    Err(err) => log::error!("error sending message: {:?}", err),
                }
            }
            show_form.set(Form::None);
        }
    };

    let form_view = move || {
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
                            Form::Options => {
                                view! { cx,
                                    <>
                                        <Options chess_board_signals=chess_board_signals submit=options_submit/>
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
    };

    view! { cx,
        <>
            <Menu show_form=show_form chess_board_signals=chess_board_signals />
            <StatusMenu show_form=show_form chess_board_signals=chess_board_signals />
            {form_view}
        </>
    }
}
