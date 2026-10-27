use leptos::prelude::*;

use crate::{
    components::{people::PeopleList, top_bar::room_title},
    entities::chess_board::signals::ChessBoardSignals,
};

/// The desktop's left column: what this room is, and who is in it
#[component]
pub fn RoomPanel(chess_board_signals: ChessBoardSignals) -> impl IntoView {
    let title = move || room_title(chess_board_signals).0;
    let room = move || {
        chess_board_signals
            .room_status()
            .with(|status| status.as_ref().map(|status| status.name()))
            .unwrap_or_default()
    };
    let eyebrow = move || {
        if chess_board_signals.is_game() {
            "Game"
        } else {
            "Free board"
        }
    };
    let count = move || {
        chess_board_signals
            .room_status()
            .with(|status| status.as_ref().map(|s| s.users_count()).unwrap_or(0))
    };

    view! {
        <aside class="hidden lg:flex flex-col gap-4">
            <div class="card p-4 flex flex-col gap-2">
                <span class="eyebrow">{eyebrow}</span>
                <span class="text-lg font-semibold">{title}</span>
                <span class="font-mono text-[13px] text-ink-300">{room}</span>
            </div>
            <div class="card p-4 flex flex-col gap-3">
                <span class="eyebrow">{move || format!("In the room · {}", count())}</span>
                <PeopleList chess_board_signals=chess_board_signals/>
            </div>
        </aside>
    }
}
