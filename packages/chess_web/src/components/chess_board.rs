use crate::components::board::BoardBackground;
use crate::components::coordinates::Coordinates;
use crate::entities::chess_board::ChessBoard as ChessBoardEntity;
use crate::entities::position::Position;
use crate::entities::stone::Stone;
use crate::handlers::mouse::{mousedown, mouseover};
use leptos::*;

#[component]
pub fn ChessBoard(cx: Scope, chess_board: ReadSignal<ChessBoardEntity>) -> impl IntoView {
    view! {
      cx,
      <chess-board
        class={move || chess_board.get().css_class() }
        on:mouseover=move |e| mouseover(e)
      >
        <BoardBackground />
        <Coordinates is_white_view={(move || chess_board.get().is_white_view)() } />
        <For
          // a function that returns the items we're iterating over; a signal is fine
          each={move || chess_board.get().stones_and_positions()}
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
