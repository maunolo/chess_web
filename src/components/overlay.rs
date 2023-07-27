use leptos::*;

use crate::{
    components::{
        forms::{Form, Forms},
        menu::Menu,
        notifications::Notifications,
        status_menu::StatusMenu,
    },
    entities::chess_board::signals::ChessBoardSignals,
    utils::{get_cookie_value, jwt::decode, SessionPayload, WindowExt},
};

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

    view! { cx,
        <>
            <Notifications chess_board_signals=chess_board_signals />
            <Menu show_form=show_form chess_board_signals=chess_board_signals />
            <StatusMenu show_form=show_form chess_board_signals=chess_board_signals />
            <Forms show_form=show_form chess_board_signals=chess_board_signals />
        </>
    }
}
