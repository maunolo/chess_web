use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::components::board::BoardBackground;
use crate::components::chess_board::ChessBoard;
use crate::components::coordinates::Coordinates;
use crate::components::overlay::Overlay;
use crate::components::trash::{Trash, TrashType};
use crate::entities::chess_board::{ChessBoard as ChessBoardEntity, ChessBoardSignals};
use crate::handlers::{interaction_end_with_websocket, interaction_move};

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    provide_meta_context(cx);

    view! { cx,
        <Stylesheet id="leptos" href="/style.css"/>
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
        <Link rel="preconnect" href="https://fonts.googleapis.com"/>
        <Link rel="preconnect" href="https://fonts.gstatic.com" crossorigin=""/>
        <Link href="https://fonts.googleapis.com/css2?family=Fira+Code:wght@300;400;500;600;700&display=swap" rel="stylesheet"/>
        <Router>
            <Routes>
                <Route
                    path=""
                    view=move |cx| {
                        view! { cx, <Home/> }
                    }
                />
            </Routes>
        </Router>
    }
}

#[component]
fn Home(cx: Scope) -> impl IntoView {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let chess_board_entity = ChessBoardEntity::new(fen);
    let (chess_board, set_chess_board) = create_signal::<ChessBoardEntity>(cx, chess_board_entity);
    let (should_render, set_should_render) = create_signal(cx, false);
    let chess_board_signals = ChessBoardSignals::new(set_chess_board, set_should_render);
    let chess_board_socket = create_rw_signal::<Option<web_sys::WebSocket>>(cx, None);

    view! { cx,
        <div
            class="flex overflow-hidden relative justify-center items-center px-4 w-screen h-screen sm:py-16 sm:px-16 md:py-16 md:px-0"
            on:touchmove=interaction_move
            on:touchend=move |e| interaction_end_with_websocket(chess_board_socket, e)
            on:mousemove=interaction_move
            on:mouseup=move |e| interaction_end_with_websocket(chess_board_socket, e)
        >
            <Show
                when=move || should_render.get()
                fallback=|cx| {
                    view! { cx,
                        <chess-board class="chessboard">
                            <BoardBackground/>
                            <Coordinates white_view=move || true/>
                            <Trash id=TrashType::Dark white_view=move || true trash=move || vec![]/>
                            <Trash id=TrashType::Light white_view=move || true trash=move || vec![]/>
                        </chess-board>
                    }
                }
            >
                <ChessBoard chess_board=chess_board/>
            </Show>
            <Overlay
                chess_board=set_chess_board
                chess_board_socket=chess_board_socket
                chess_board_signals=chess_board_signals
            />
        </div>
    }
}
