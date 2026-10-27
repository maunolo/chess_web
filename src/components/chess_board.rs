use crate::components::board::BoardBackground;
use crate::components::coordinates::Coordinates;
use crate::components::trash::{Trash, TrashType};
use crate::entities::chess_board::signals::{ChessBoardSignals, StoneSignal};
use crate::entities::chess_board::turns::Turn;
use crate::entities::position::Position;
use crate::entities::stone::{Color, Kind};
use crate::handlers::pointer_down;
use leptos::prelude::*;

fn on_square(class: &str, position: &Position) -> String {
    format!("{} square-{}", class, position.to_string())
}

#[component]
pub fn ChessBoard(chess_board_signals: ChessBoardSignals) -> impl IntoView {
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

    let last_move_squares = move || {
        chess_board_signals
            .last_move()
            .get()
            .map(|(from, to)| [from, to].into_iter().flatten().collect::<Vec<_>>())
            .unwrap_or_default()
    };
    let selected = move || chess_board_signals.selected().get();
    let selected_square = move || selected().and_then(|selected| selected.from);
    // where the selected piece can go, when the rules are enforced
    let move_hints = move || {
        let Some(selected) = selected() else {
            return vec![];
        };
        let Some(from) = selected.from else {
            return vec![];
        };
        if !chess_board_signals.validation() {
            return vec![];
        }

        chess_board_signals.chess_board().with(|board| {
            let side_to_move = match board.turn {
                Turn::White => "l",
                Turn::Black => "d",
            };
            if !selected.piece.starts_with(side_to_move) {
                return vec![];
            }

            let mut hints: Vec<(Position, bool)> = board
                .possible_moves(&from)
                .into_iter()
                .map(|to| {
                    let capture = board.stone_at(to.x, to.y).is_some();
                    (to, capture)
                })
                .collect();
            hints.sort_by_key(|(position, _)| (position.y, position.x));
            hints
        })
    };
    let king_in_check = move || {
        if !chess_board_signals.validation() {
            return None;
        }

        chess_board_signals.chess_board().with(|board| {
            if !board.is_in_check() {
                return None;
            }
            let color = match board.turn {
                Turn::White => Color::Light,
                Turn::Black => Color::Dark,
            };
            board
                .stones_and_positions_iter()
                .find(|(_, stone)| stone.kind() == Kind::King && stone.color() == color)
                .map(|(position, _)| position)
        })
    };

    let piece_view = move |(key, stone_signal): (String, RwSignal<StoneSignal>)| {
        let position = move || stone_signal.get().position().map(|p| p.to_string());
        let stone = move || stone_signal.get().stone();
        let dragging_class = move || {
            if stone_signal.get().is_dragging() {
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
        view! {
            <div
                class=class
                on:pointerdown=move |e| pointer_down(chess_board_signals, e)
                on:dragstart=move |e| e.prevent_default()
                data-square=move || position().unwrap_or("deleted".to_string())
                data-piece=move || stone().image_class()
                data-key=key
                data-deleted=move || format!("{}", stone_signal.get().is_deleted())
            ></div>
        }
    };

    view! {
        <chess-board class=css_class id="chessboard">
            <BoardBackground/>
            <Coordinates white_view=white_view/>
            <For
                each=last_move_squares
                key=|position| position.to_string()
                children=|position| view! { <div class=on_square("last-move", &position)></div> }
            />
            {move || {
                selected_square()
                    .map(|position| view! { <div class=on_square("selected", &position)></div> })
            }}
            {move || {
                king_in_check()
                    .map(|position| view! { <div class=on_square("in-check", &position)></div> })
            }}
            // Keyed on the signal too: a `/sync_board` replaces every stone's
            // signal, and a piece still on its square must bind to the new one
            <For
                each=stones_signals
                key=|(key, stone_signal)| (key.clone(), *stone_signal)
                children=piece_view
            />
            <For
                each=move_hints
                key=|(position, capture)| (position.to_string(), *capture)
                children=|(position, capture)| {
                    let class = if capture { "move-hint capture" } else { "move-hint" };
                    view! { <div class=on_square(class, &position)></div> }
                }
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
