use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use cfg_if::cfg_if;

use crate::components::chess_board::ChessBoard;
use crate::entities::chess_board::ChessBoard as ChessBoardEntity;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    provide_meta_context(cx);

    view! {
        cx,
        <Stylesheet id="leptos" href="/style.css"/>
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
        <Router>
            <Routes>
                <Route path="" view=  move |cx| view! { cx, <Home/> }/>
            </Routes>
        </Router>
    }
}

#[component]
fn Home(cx: Scope) -> impl IntoView {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let mut chessboard_entity = ChessBoardEntity::new(fen);
    chessboard_entity.flip();
    let (chessboard, set_chessboard) = create_signal(cx, chessboard_entity);

    cfg_if! {
      if #[cfg(feature = "ssr")] {
        let mousemove = move |_| {};
        let mouseup = move |_| {};
      } else {
        use crate::handlers::mouse::{mousemove, mouseup};
      }
    }

    view! { cx,
      <div
        class="flex overflow-hidden relative justify-center items-center px-4 w-screen h-screen md:py-8 md:px-0"
        // ontouchmove={Callback::from(mousemove)}
        // ontouchup={Callback::from(mouseup)}
        on:mousemove=move |e| mousemove(e)
        on:mouseup=move |e| mouseup(e)
      >
        <ChessBoard chessboard=chessboard />
        <button class="absolute top-2 left-2 z-30 py-2 px-10 rounded bg-neutral-300 hover:bg-neutral-400" on:click=move |_| {
            set_chessboard.update(|cb| cb.flip());
        } >"Flip the Board"</button>
      </div>
    }
}
