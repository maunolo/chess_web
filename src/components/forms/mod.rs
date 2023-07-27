pub mod join;
pub mod options;
pub mod username;

use crate::entities::chess_board::signals::ChessBoardSignals;
use crate::utils::WindowExt;
use leptos::*;

use self::{join::Join, options::Options, username::Username};

#[derive(Clone)]
#[allow(dead_code)]
pub enum Form {
    None,
    Join,
    Username,
    Options,
}

fn set_username(username: &str, chess_board_signals: ChessBoardSignals) {
    let window = web_sys::window().unwrap();
    window.post_user_name(username, chess_board_signals);
}

#[component]
pub fn Forms(
    cx: Scope,
    chess_board_signals: ChessBoardSignals,
    show_form: RwSignal<Form>,
) -> impl IntoView {
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

    let view = move || match show_form.get() {
        Form::Join => {
            view! { cx,
                <div class="z-40 flex absolute w-full h-full justify-center items-center bg-neutral-900/30">
                    <Join submit=join_submit/>
                </div>
            }
        }
        Form::Username => {
            view! { cx,
                <div class="z-40 flex absolute w-full h-full justify-center items-center bg-neutral-900/30">
                    <Username submit=username_submit/>
                </div>
            }
        }
        Form::Options => {
            view! { cx,
                <div class="z-40 flex absolute w-full h-full justify-center items-center bg-neutral-900/30">
                    <Options chess_board_signals=chess_board_signals submit=options_submit/>
                </div>
            }
        }
        _ => {
            view! { cx,
                <div class="hidden"></div>
            }
        }
    };

    view
}
