use leptos::prelude::*;

use crate::{
    entities::chess_board::signals::ChessBoardSignals,
    i18n::{self, Key},
};

/// The free board's rules: enforce legal moves, and alternate turns
#[component]
pub fn Options<F>(chess_board_signals: ChessBoardSignals, submit: F) -> impl IntoView
where
    F: Fn(web_sys::SubmitEvent) -> () + 'static,
{
    let lang = i18n::current();
    let validation = move || {
        chess_board_signals
            .room_status()
            .get()
            .map(|rs| rs.options().validation())
            .unwrap_or(false)
    };
    let sync = move || {
        chess_board_signals
            .room_status()
            .get()
            .map(|rs| rs.options().sync())
            .unwrap_or(false)
    };

    view! {
        <form class="dialog" on:submit=submit>
            <span class="dialog-title">{lang.t(Key::BoardOptions)}</span>
            <label class="flex items-center gap-3 cursor-pointer">
                <span class="switch">
                    <input type="checkbox" name="validation" checked=validation/>
                    <span class="slider round"></span>
                </span>
                <span class="flex flex-col">
                    <span class="text-sm font-medium">{lang.t(Key::Validation)}</span>
                    <span class="text-xs text-ink-300">{lang.t(Key::ValidationHint)}</span>
                </span>
            </label>
            <label class="flex items-center gap-3 cursor-pointer">
                <span class="switch">
                    <input type="checkbox" name="sync" checked=sync/>
                    <span class="slider round"></span>
                </span>
                <span class="flex flex-col">
                    <span class="text-sm font-medium">{lang.t(Key::Turns)}</span>
                    <span class="text-xs text-ink-300">{lang.t(Key::TurnsHint)}</span>
                </span>
            </label>
            <button class="btn btn-primary" type="submit">
                {lang.t(Key::Apply)}
            </button>
        </form>
    }
}
