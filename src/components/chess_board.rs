use crate::components::board::BoardBackground;
use crate::components::coordinates::Coordinates;
use crate::components::trash::{Trash, TrashType};
use crate::entities::chess_board::ChessBoard as ChessBoardEntity;
use crate::entities::position::Position;
use crate::entities::stone::Stone;
use crate::handlers::interaction_start;
use leptos::*;

#[component]
pub fn ChessBoard(cx: Scope, chess_board: ReadSignal<ChessBoardEntity>) -> impl IntoView {
    let css_class = move || chess_board.with(|c| c.css_class());
    let white_view = move || chess_board.with(|c| c.white_view());
    let stones_and_positions = move || chess_board.with(|c| c.stones_and_positions());
    let trash = move || chess_board.with(|c| c.deleted_stones());

    view! { cx,
        <chess-board class=css_class id="chessboard">
            <BoardBackground/>
            <Coordinates white_view=white_view/>
            <For
                each=stones_and_positions
                key=move |(position, stone)| { format!("{}-{}", position.to_string(), stone.image_class) }
                view=move |cx, (position, stone): (Position, Stone)| {
                    view! { cx,
                        <div
                            class=format!("piece {} {}", stone.image_class.clone(), position.css_class())
                            on:mousedown=interaction_start
                            on:touchstart=interaction_start
                            on:dragstart=move |e| e.prevent_default()
                            data-square=position.to_string()
                            data-piece=stone.image_class.clone()
                        ></div>
                    }
                }
            />
            <Trash id=TrashType::Dark white_view=white_view trash=trash/>
            <Trash id=TrashType::Light white_view=white_view trash=trash/>
        </chess-board>
    }
}
