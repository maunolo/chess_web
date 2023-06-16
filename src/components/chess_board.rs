use crate::components::board::BoardBackground;
use crate::components::coordinates::Coordinates;
use crate::components::trash::Trash;
use crate::entities::chess_board::ChessBoard as ChessBoardEntity;
use crate::entities::position::Position;
use crate::entities::stone::Stone;
use leptos::*;

use cfg_if::cfg_if;

#[component]
pub fn ChessBoard(cx: Scope, chessboard: ReadSignal<ChessBoardEntity>) -> impl IntoView {
    cfg_if! {
      if #[cfg(feature = "ssr")] {
        let mousedown = move |_| {};
        let touchstart = move |_| {};
      } else {
        use crate::handlers::mouse::{mousedown, touchstart};
      }
    }

    view! {
      cx,
      <chess-board
        class=move || chessboard.with(|cb| cb.css_class())
        id="chessboard"
      >
        <BoardBackground />
        <Coordinates white_view={ move || chessboard.with(|cb| cb.is_white_view)} />
        <For
            // a function that returns the items we're iterating over; a signal is fine
            each=move || chessboard.get().stones_and_positions()
            // a unique key for each item
            key=move |(position, stone)| {
                format!("{}-{}", position.to_string(), stone.image_class)
            }
            // renders each item to a view
            view=move |cx, (position, stone): (Position, Stone)| {
                view! {
                    cx,
                    <div
                        class={format!("piece {} {}", stone.image_class.clone(), position.css_class())}
                    on:mousedown=move |e| mousedown(e)
                        on:touchstart=move |e| touchstart(e)
                        on:dragstart=move |e| e.prevent_default()
                        data-square={position.to_string()}
                    data-piece={stone.image_class.clone()}
                    ></div>
                }
            }
        />
        <Trash
            id={"dark".to_string()}
            white_view={ move || chessboard.with(|cb| cb.is_white_view)}
            chessboard=chessboard
        />
        <Trash
            id={"light".to_string()}
            white_view={ move || chessboard.with(|cb| cb.is_white_view)}
            chessboard=chessboard
        />
      </chess-board>
    }
}
