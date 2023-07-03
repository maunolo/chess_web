use leptos::*;

use crate::{
    components::forms::{join::Join, username::Username},
    entities::{chess_board::ChessBoardSignals, room::User},
    utils::{get_cookie_value, jwt::decode, SessionPayload, WindowExt},
};

#[allow(unused_variables)]
fn toggle_menu(menu_timeout_id: RwSignal<Option<i32>>, show_menu: bool) {
    let window = web_sys::window().unwrap();
    let sub_menu = window
        .document()
        .unwrap()
        .get_element_by_id("sub-menu")
        .unwrap();

    if show_menu {
        sub_menu.set_class_name("sub-menu sub-menu--is-active")
    } else {
        if let Some(id) =
            window.set_timeout_callback(move || sub_menu.set_class_name("sub-menu"), 250)
        {
            menu_timeout_id.set(Some(id))
        };
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

fn set_username(username: &str, chess_board_signals: ChessBoardSignals) {
    let window = web_sys::window().unwrap();
    window.post_user_name(username, chess_board_signals);
}

fn get_user_payload() -> Option<SessionPayload> {
    if let Some(session_token) = get_cookie_value("session_token") {
        let Ok(token) = decode::<SessionPayload>(&session_token) else {
            log::debug!("Session token is not a valid JWT: {}", session_token);
            return None;
        };
        Some(token.claims().clone())
    } else {
        None
    }
}

#[component]
#[allow(unused_variables)]
pub fn Overlay(cx: Scope, chess_board_signals: ChessBoardSignals) -> impl IntoView {
    let (show_form, set_show_form) = create_signal(cx, Form::None);
    let (show_menu, set_show_menu) = create_signal(cx, false);
    let (show_status_menu, set_show_status_menu) = create_signal(cx, false);
    let menu_timeout_id = create_rw_signal::<Option<i32>>(cx, None);
    let status_menu_timeout_id = create_rw_signal::<Option<i32>>(cx, None);

    create_effect(cx, move |_| {
        use crate::utils::WindowExt;

        if let Some(username) = get_user_payload().map(|payload| payload.name) {
            chess_board_signals.start_websocket();
        } else {
            let window = web_sys::window().unwrap();
            window.set_timeout_callback(move || set_show_form.set(Form::Username), 0);
        };
    });

    let join = move |_| {
        set_show_form.set(Form::Join);
    };

    let username = move |_| {
        set_show_form.set(Form::Username);
    };

    let reset = move |_| {
        if let Some(socket) = chess_board_signals.socket().get().as_ref() {
            match socket.send_with_str("/reset") {
                Ok(_) => log::debug!("message successfully sent: {:?}", "/reset"),
                Err(err) => log::debug!("error sending message: {:?}", err),
            }
        }
    };

    let toggle_menu = move |_| {
        set_show_menu.update(|show_menu| {
            clear_timeout(menu_timeout_id.get());
            toggle_menu(menu_timeout_id, !*show_menu);

            *show_menu = !*show_menu;
        });
    };

    let toggle_status_menu = move |_| {
        set_show_status_menu.update(|show_status_menu| {
            // clear_timeout(sub_menu_timeout_id.get());
            // toggle_status_menu(sub_menu_timeout_id, !*show_status_menu);

            *show_status_menu = !*show_status_menu;
        });
    };

    let start_websocket = move |_| {
        chess_board_signals.start_websocket();
    };

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
            set_username(&username, chess_board_signals);
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

    let status_menu_css = move || {
        let is_active = show_status_menu.get();
        let needs_refresh = chess_board_signals.socket().get().is_none();
        let mut class = "status-menu".to_string();

        if needs_refresh {
            class.push_str(" status-menu--refresh");
        }
        if is_active {
            class.push_str(" status-menu--is-active");
        }

        class
    };

    let status_menu_btn_css = move || {
        let is_active = show_status_menu.get();
        let is_online = chess_board_signals.socket().with(|s| s.is_some());
        let mut class = "status-menu-btn".to_string();

        if is_online && is_active {
            class.push_str(" status-menu-btn--is-online--is-active");
        } else if is_online {
            class.push_str(" status-menu-btn--is-online");
        } else if is_active {
            class.push_str(" status-menu-btn--is-active");
        }

        class
    };

    let status_refresh_btn_css = move || {
        let needs_refresh = chess_board_signals.socket().get().is_none();
        let mut class = "status-refresh-btn".to_string();

        if needs_refresh {
            class.push_str(" status-refresh-btn--is-active");
        }

        class
    };

    let status_menu_style = move || {
        if show_status_menu.get() {
            let users_count = chess_board_signals
                .room_status()
                .with(|status| status.as_ref().map(|s| s.users_count()).unwrap_or(0));
            let height = 3.75 + (2 * users_count) as f64 + 0.5;

            format!("height: {}rem;", height)
        } else {
            "".to_string()
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
                        move || chess_board_signals.room_status().with(|status| status.as_ref().map(|s| s.name().clone()).unwrap_or("Chess".to_owned()))
                    }</h1>
                </div>
                <div class="sub-menu" id="sub-menu">
                    <button
                        class="sub-menu-item"
                        on:click=move |_| { chess_board_signals.chess_board().update(|cb| cb.flip()) }
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
            <div class=status_menu_css style=status_menu_style>
                <div class="status-menu-header">
                    <button
                        class=status_refresh_btn_css
                        on:click=start_websocket
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" version="1.1" id="Capa_1" x="0px" y="0px" viewBox="0 0 513.806 513.806" style="enable-background:new 0 0 513.806 513.806;" xml:space="preserve" width="512" height="512">
                            <g>
                                <path d="M66.074,228.731C81.577,123.379,179.549,50.542,284.901,66.045c35.944,5.289,69.662,20.626,97.27,44.244l-24.853,24.853   c-8.33,8.332-8.328,21.84,0.005,30.17c3.999,3.998,9.423,6.245,15.078,6.246h97.835c11.782,0,21.333-9.551,21.333-21.333V52.39   c-0.003-11.782-9.556-21.331-21.338-21.329c-5.655,0.001-11.079,2.248-15.078,6.246L427.418,65.04   C321.658-29.235,159.497-19.925,65.222,85.835c-33.399,37.467-55.073,83.909-62.337,133.573   c-2.864,17.607,9.087,34.202,26.693,37.066c1.586,0.258,3.188,0.397,4.795,0.417C50.481,256.717,64.002,244.706,66.074,228.731z"/>
                                <path d="M479.429,256.891c-16.108,0.174-29.629,12.185-31.701,28.16C432.225,390.403,334.253,463.24,228.901,447.738   c-35.944-5.289-69.662-20.626-97.27-44.244l24.853-24.853c8.33-8.332,8.328-21.84-0.005-30.17   c-3.999-3.998-9.423-6.245-15.078-6.246H43.568c-11.782,0-21.333,9.551-21.333,21.333v97.835   c0.003,11.782,9.556,21.331,21.338,21.329c5.655-0.001,11.079-2.248,15.078-6.246l27.733-27.733   c105.735,94.285,267.884,85.004,362.17-20.732c33.417-37.475,55.101-83.933,62.363-133.615   c2.876-17.605-9.064-34.208-26.668-37.084C482.655,257.051,481.044,256.91,479.429,256.891z"/>
                            </g>
                        </svg>
                    </button>

                    <button
                        class=status_menu_btn_css
                        on:click=toggle_status_menu
                    >
                        <div class="circle"><div class="inner-circle"/></div>
                    </button>
                </div>
                <div class="status-sub-menu">
                    <ul>
                        <For
                            each=move || {
                                chess_board_signals.room_status().with(|status| status.as_ref().map(|s| s.users()).unwrap_or(vec![]))
                            }
                            key=move |user: &User| format!("{}:{}", user.id(), user.username())
                            view=move |cx, user: User| {
                                let user_id = format!("user-{}", user.id());
                                if user.id() == get_user_payload().map(|p| p.sub).unwrap_or_default() {
                                    view! {
                                        cx,
                                        <li class="current-user" id=user_id>
                                            <span>
                                                {user.username()}
                                            </span>
                                            <span class="you">{"(You)"}</span>
                                            <button on:click=username>
                                                <svg xmlns="http://www.w3.org/2000/svg" id="Layer_1" data-name="Layer 1" viewBox="0 0 24 24" width="512" height="512">
                                                    <polygon points="14.604 5.687 0 20.29 0 24 3.71 24 18.313 9.396 14.604 5.687"/>
                                                    <path d="M23.232.768a2.624,2.624,0,0,0-3.71,0l-3.5,3.505,3.709,3.709,3.5-3.5A2.624,2.624,0,0,0,23.232.768Z"/>
                                                </svg>
                                            </button>
                                        </li>
                                    }
                                } else {
                                    view! {
                                        cx,
                                        <li id=user_id>
                                            <span>
                                                {user.username()}
                                            </span>
                                        </li>
                                    }
                                }
                            }
                        />
                    </ul>
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
