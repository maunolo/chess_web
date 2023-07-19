use crate::components::board::BoardBackground;
use crate::components::coordinates::Coordinates;
use crate::components::trash::{Trash, TrashType};
use crate::entities::chess_board::{ChessBoardSignals, StoneSignal};
use crate::handlers::interaction_start;
use leptos::*;

#[component]
pub fn ChessBoard(cx: Scope, chess_board_signals: ChessBoardSignals) -> impl IntoView {
    let css_class = move || chess_board_signals.chess_board().with(|c| c.css_class());
    let white_view = move || chess_board_signals.chess_board().with(|c| c.white_view());
    let stones_signals = move || {
        chess_board_signals
            .stones_signals()
            .get()
            .board_stones()
            .clone()
            .into_iter()
    };
    let deleted_stones_signals = move || {
        chess_board_signals
            .stones_signals()
            .get()
            .deleted_stones()
            .clone()
    };

    let piece_view = move |cx, (key, stone_signal): (String, RwSignal<StoneSignal>)| {
        let position = move || stone_signal().position().map(|p| p.to_string());
        let stone = move || stone_signal().stone();
        let dragging_class = move || {
            if stone_signal().is_dragging() {
                "dragging".to_string()
            } else {
                "".to_string()
            }
        };
        let class = move || {
            format!(
                "piece {} {} {}",
                stone().image_class(),
                position()
                    .map(|s| format!("square-{}", s))
                    .unwrap_or("".to_string()),
                dragging_class()
            )
        };
        view! { cx,
            <div
                class=class
                on:mousedown=move |e| interaction_start(chess_board_signals, e)
                on:touchstart=move |e| interaction_start(chess_board_signals, e)
                on:dragstart=move |e| e.prevent_default()
                data-square=move || position().unwrap_or("deleted".to_string())
                data-piece=move || stone().image_class()
                data-key=key
                data-deleted=move || format!("{}", stone_signal().is_deleted())
            ></div>
        }
    };

    view! { cx,
        <chess-board class=css_class id="chessboard">
            <BoardBackground/>
            <Coordinates white_view=white_view/>
            <For
                each=stones_signals
                key=move |(key, _)| key.to_string()
                view=piece_view
            />
            <Trash
                chess_board_signals=chess_board_signals
                id=TrashType::Dark
                white_view=white_view
                trash=deleted_stones_signals
            />
            <Trash
                chess_board_signals=chess_board_signals
                id=TrashType::Light
                white_view=white_view
                trash=deleted_stones_signals
            />
        </chess-board>
    }
}
