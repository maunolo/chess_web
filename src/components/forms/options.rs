use leptos::prelude::*;

use crate::entities::chess_board::signals::ChessBoardSignals;

/// The free board's rules: enforce legal moves, and alternate turns
#[component]
pub fn Options<F>(chess_board_signals: ChessBoardSignals, submit: F) -> impl IntoView
where
    F: Fn(web_sys::SubmitEvent) -> () + 'static,
{
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
            <span class="dialog-title">"Board options"</span>
            <label class="flex items-center gap-3 cursor-pointer">
                <span class="switch">
                    <input type="checkbox" name="validation" checked=validation/>
                    <span class="slider round"></span>
                </span>
                <span class="flex flex-col">
                    <span class="text-sm font-medium">"Validation"</span>
                    <span class="text-xs text-ink-300">"Only legal moves, with check and mate"</span>
                </span>
            </label>
            <label class="flex items-center gap-3 cursor-pointer">
                <span class="switch">
                    <input type="checkbox" name="sync" checked=sync/>
                    <span class="slider round"></span>
                </span>
                <span class="flex flex-col">
                    <span class="text-sm font-medium">"Turns"</span>
                    <span class="text-xs text-ink-300">"White and black alternate"</span>
                </span>
            </label>
            <button class="btn btn-primary" type="submit">
                "Apply"
            </button>
        </form>
    }
}
