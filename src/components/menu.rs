use leptos::*;

use crate::{
    components::{
        forms::Form,
        overlay::{clear_timeout, toggle_sub_menu},
    },
    entities::chess_board::signals::ChessBoardSignals,
};

#[component]
pub fn Menu(show_form: RwSignal<Form>, chess_board_signals: ChessBoardSignals) -> impl IntoView {
    let show_menu = create_rw_signal(false);
    let menu_timeout_id = create_rw_signal::<Option<i32>>(None);

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

    let toggle_menu = move |_| {
        show_menu.update(|show_menu| {
            clear_timeout(menu_timeout_id.get());
            toggle_sub_menu(menu_timeout_id, !*show_menu, "sub-menu");

            *show_menu = !*show_menu;
        });
    };

    let reset = move |_| {
        if let Some(socket) = chess_board_signals.socket().get().as_ref() {
            match socket.send_with_str("/reset") {
                Ok(_) => {}
                Err(err) => log::error!("Error sending message: {:?}", err),
            }
        }
    };

    let undo = move |_| {
        if let Some(socket) = chess_board_signals.socket().get().as_ref() {
            match socket.send_with_str("/undo") {
                Ok(_) => {}
                Err(err) => log::error!("Error sending message: {:?}", err),
            }
        }
    };

    let redo = move |_| {
        if let Some(socket) = chess_board_signals.socket().get().as_ref() {
            match socket.send_with_str("/redo") {
                Ok(_) => {}
                Err(err) => log::error!("Error sending message: {:?}", err),
            }
        }
    };

    let join = move |_| {
        show_form.set(Form::Join);
    };

    let options = move |_| {
        show_form.set(Form::Options);
    };

    view! {
        <div class=menu_css>
            <div class="menu-header">
                <button
                    class=menu_btn_css
                    on:click=toggle_menu
                    aria-label="Menu"
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
                <button
                    class="sub-menu-item"
                    on:click=options
                >
                    "Options"
                </button>
                <div class="split-button"
                >
                    <button
                        class="split-button-item"
                        on:click=undo
                    >
                        "Undo"
                    </button>
                    <button
                        class="split-button-item"
                        on:click=redo
                    >
                        "Redo"
                    </button>
                </div>
            </div>
        </div>
    }
}
