use leptos::prelude::*;

use crate::{
    components::{
        forms::Form,
        overlay::{clear_timeout, toggle_sub_menu},
    },
    entities::{chess_board::signals::ChessBoardSignals, protocol::ClientMsg},
};

#[component]
pub fn Menu(show_form: RwSignal<Form>, chess_board_signals: ChessBoardSignals) -> impl IntoView {
    let show_menu = RwSignal::new(false);
    let menu_timeout_id = RwSignal::<Option<i32>>::new(None);
    // a first tap on "Resign" asks for a second one
    let confirm_resign = RwSignal::new(false);

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
        confirm_resign.set(false);
        show_menu.update(|show_menu| {
            clear_timeout(menu_timeout_id.get_untracked());
            toggle_sub_menu(menu_timeout_id, !*show_menu, "sub-menu");

            *show_menu = !*show_menu;
        });
    };

    let send = move |msg: ClientMsg| chess_board_signals.send(&msg);

    let is_game = move || chess_board_signals.is_game();
    let playing = move || {
        chess_board_signals.my_seat().is_some()
            && chess_board_signals
                .game()
                .with(|game| game.as_ref().is_some_and(|game| game.result.is_none()))
    };
    // the menu's height follows its number of rows
    let menu_style = move || {
        let rows = match (is_game(), playing()) {
            (false, _) | (true, true) => 6,
            (true, false) => 4,
        };
        format!("--menu-rows: {rows}")
    };

    let draw_offer = move || {
        let seat = chess_board_signals.my_seat()?;
        let offer = chess_board_signals
            .game()
            .with(|game| game.as_ref().and_then(|game| game.draw_offer))?;
        Some(offer == seat)
    };
    let draw_label = move || match draw_offer() {
        Some(true) => "Draw offered",
        Some(false) => "Accept draw",
        None => "Offer draw",
    };
    let draw = move |_| match draw_offer() {
        Some(false) => send(ClientMsg::AcceptDraw),
        _ => send(ClientMsg::OfferDraw),
    };
    let resign = move |_| {
        if confirm_resign.get_untracked() {
            confirm_resign.set(false);
            send(ClientMsg::Resign);
        } else {
            confirm_resign.set(true);
        }
    };

    view! {
        <div class=menu_css style=menu_style>
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
                <button class="sub-menu-item" on:click=move |_| show_form.set(Form::NewGame)>
                    "New game"
                </button>
                <button class="sub-menu-item" on:click=move |_| show_form.set(Form::Join)>
                    "Join"
                </button>
                <Show
                    when=is_game
                    fallback=move || view! {
                        <button class="sub-menu-item" on:click=move |_| send(ClientMsg::Reset)>
                            "Reset"
                        </button>
                        <button class="sub-menu-item" on:click=move |_| show_form.set(Form::Options)>
                            "Options"
                        </button>
                        <div class="split-button">
                            <button class="split-button-item" on:click=move |_| send(ClientMsg::Undo)>
                                "Undo"
                            </button>
                            <button class="split-button-item" on:click=move |_| send(ClientMsg::Redo)>
                                "Redo"
                            </button>
                        </div>
                    }
                >
                    <button class="sub-menu-item" on:click=move |_| show_form.set(Form::Invite)>
                        "Invite"
                    </button>
                    <Show when=playing>
                        <button class="sub-menu-item" on:click=draw>
                            {draw_label}
                        </button>
                        <button class="sub-menu-item" on:click=resign>
                            {move || if confirm_resign.get() { "Confirm resign" } else { "Resign" }}
                        </button>
                    </Show>
                </Show>
            </div>
        </div>
    }
}
