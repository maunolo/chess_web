use crate::components::board::BoardBackground;
use crate::components::coordinates::Coordinates;
use crate::entities::chess_board::ChessBoard as ChessBoardEntity;
use crate::entities::position::Position;
use crate::entities::stone::Stone;
use leptos::*;

use cfg_if::cfg_if;

#[component]
pub fn ChessBoard(cx: Scope, chess_board: ReadSignal<ChessBoardEntity>) -> impl IntoView {
    cfg_if! {
      if #[cfg(feature = "ssr")] {
        let mousedown = move |_| {};
        let mouseover = move |_| {};
      } else {
        use crate::handlers::mouse::{mousedown, mouseover};
      }
    }

    view! {
      cx,
      <chess-board
        class=chess_board.with(|cb| cb.css_class())
        on:mouseover=move |e| mouseover(e)
      >
        <BoardBackground />
        <Coordinates is_white_view=chess_board.with(|cb| cb.is_white_view) />
        <For
          // a function that returns the items we're iterating over; a signal is fine
          each={move || chess_board.with(|cb| cb.stones_and_positions()) }
          // a unique key for each item
          key=|(_position, stone)| stone.name.clone()
          // renders each item to a view
          view=move |cx, (position, stone): (Position, Stone)| {
            view! {
              cx,
              <div
                class={format!("piece {} {}", stone.image_class, position.css_class())}
                on:mousedown=move |e| mousedown(e)
                on:dragstart=move |e| e.prevent_default()
                data-square={position.to_string()}
              ></div>
            }
          }
        />
      </chess-board>
    }
}
