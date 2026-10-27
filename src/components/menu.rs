use leptos::prelude::*;

use crate::{
    components::{forms::Form, icons::Icon},
    entities::chess_board::signals::ChessBoardSignals,
};

/// The phone's menu, under the hamburger: everything that isn't a move
#[component]
pub fn Menu(
    show_form: RwSignal<Form>,
    chess_board_signals: ChessBoardSignals,
    open: RwSignal<bool>,
) -> impl IntoView {
    let class = move || {
        if open.get() {
            "dropdown dropdown--left dropdown--open"
        } else {
            "dropdown dropdown--left"
        }
    };
    let open_form = move |form: Form| {
        open.set(false);
        show_form.set(form);
    };
    let flip = move |_| {
        open.set(false);
        chess_board_signals.chess_board().update(|board| board.flip());
    };
    let free_board = move || !chess_board_signals.is_game();

    view! {
        <div class=class>
            <button class="dropdown-item" on:click=move |_| open_form(Form::NewGame)>
                {Icon::NewGame.view()}
                "New game"
            </button>
            <button class="dropdown-item" on:click=move |_| open_form(Form::Join)>
                {Icon::Join.view()}
                "Join room"
            </button>
            <button class="dropdown-item" on:click=move |_| open_form(Form::Invite)>
                {Icon::Invite.view()}
                "Invite"
            </button>
            <Show when=free_board>
                <button class="dropdown-item" on:click=move |_| open_form(Form::Options)>
                    {Icon::Options.view()}
                    "Board options"
                </button>
            </Show>
            <button class="dropdown-item" on:click=flip>
                {Icon::Flip.view()}
                "Flip board"
            </button>
            <button class="dropdown-item" on:click=move |_| open_form(Form::Username)>
                {Icon::Name.view()}
                "Change name"
            </button>
        </div>
    }
}
