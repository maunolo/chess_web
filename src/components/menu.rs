use leptos::prelude::*;

use crate::{
    components::{forms::Form, icons::Icon},
    entities::chess_board::signals::ChessBoardSignals,
    i18n::{self, Key},
};

/// The phone's menu, under the hamburger: everything that isn't a move
#[component]
pub fn Menu(
    show_form: RwSignal<Form>,
    chess_board_signals: ChessBoardSignals,
    open: RwSignal<bool>,
) -> impl IntoView {
    let lang = i18n::current();
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
                {lang.t(Key::NewGame)}
            </button>
            <button class="dropdown-item" on:click=move |_| open_form(Form::Join)>
                {Icon::Join.view()}
                {lang.t(Key::JoinRoom)}
            </button>
            <button class="dropdown-item" on:click=move |_| open_form(Form::Invite)>
                {Icon::Invite.view()}
                {lang.t(Key::Invite)}
            </button>
            <Show when=free_board>
                <button class="dropdown-item" on:click=move |_| open_form(Form::Options)>
                    {Icon::Options.view()}
                    {lang.t(Key::BoardOptions)}
                </button>
            </Show>
            <button class="dropdown-item" on:click=flip>
                {Icon::Flip.view()}
                {lang.t(Key::FlipBoard)}
            </button>
            <button class="dropdown-item" on:click=move |_| open_form(Form::Username)>
                {Icon::Name.view()}
                {lang.t(Key::ChangeName)}
            </button>
        </div>
    }
}
