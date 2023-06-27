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
            <div class="pointer-events-none flex flex-none gap-2 absolute top-0 z-30 py-2 px-2 w-full justify-center sm:w-auto sm:h-full sm:left-0 sm:flex-col sm:top-auto">
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
