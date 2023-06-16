use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use cfg_if::cfg_if;

use crate::components::board::BoardBackground;
use crate::components::chess_board::ChessBoard;
use crate::components::coordinates::Coordinates;
use crate::components::overlay::Overlay;
use crate::components::trash::{Trash, TrashType};
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
    let chessboard_entity = ChessBoardEntity::new(fen);
    // chessboard_entity.flip();
    let (chessboard, set_chessboard) = create_signal(cx, chessboard_entity);
    #[allow(unused_variables)]
    let (should_render, set_should_render) = create_signal(cx, false);

    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let mousemove = move |_| {};
            let mouseup = move |_| {};
            let touchmove = move |_| {};
            let touchend = move |_| {};
            let reset = move |_| {};

            let chess_board_socket = None;
        } else {
            use crate::handlers::mouse::{mousemove, touchmove};
            use crate::handlers::mouse;

            let chess_board_socket = crate::client::websockets::chess_board::start_websocket(set_chessboard, set_should_render).ok();

            let clone_ws = chess_board_socket.clone();
            let mouseup = move |e| {
                if let Ok((piece_data, (old_pos, new_pos))) = mouse::mouseup(e) {
                    if let Some(socket) = clone_ws.as_ref() {
                        let msg = format!("/move {} {} {}", piece_data, old_pos, new_pos);

                        match socket.send_with_str(&msg) {
                            Ok(_) => log::debug!("message successfully sent: {:?}", msg),
                            Err(err) => log::debug!("error sending message: {:?}", err),
                        }
                    }
                };
            };

            let clone_ws = chess_board_socket.clone();
            let touchend = move |e| {
                if let Ok((piece_data, (old_pos, new_pos))) = mouse::touchend(e) {
                    if let Some(socket) = clone_ws.as_ref() {
                        let msg = format!("/move {} {} {}", piece_data, old_pos, new_pos);

                        match socket.send_with_str(&msg) {
                            Ok(_) => log::debug!("message successfully sent: {:?}", msg),
                            Err(err) => log::debug!("error sending message: {:?}", err),
                        }
                    }
                };
            };

            let clone_ws = chess_board_socket.clone();
            let reset = move |_| {
                if let Some(socket) = clone_ws.as_ref() {
                    match socket.send_with_str("/reset") {
                        Ok(_) => log::debug!("message successfully sent: {:?}", "/reset"),
                        Err(err) => log::debug!("error sending message: {:?}", err),
                    }
                }
            };
        }
    }

    let (chess_board_socket, _) = create_signal(cx, chess_board_socket);

    view! { cx,
        <div
            class="flex overflow-hidden relative justify-center items-center px-4 w-screen h-screen sm:py-16 sm:px-16 md:py-16 md:px-0"
            on:touchmove=touchmove
            on:touchend=touchend
            on:mousemove=mousemove
            on:mouseup=mouseup
        >
            <Show
                when=move || should_render.get()
                fallback=|cx| {
                    view! {
                        cx,
                        <chess-board class="chessboard">
                            <BoardBackground/>
                            <Coordinates white_view=move || true />
                            <Trash id=TrashType::Dark white_view=move || true trash=move || vec![] />
                            <Trash id=TrashType::Light white_view=move || true trash=move || vec![] />
                        </chess-board>
                    }
                }
            >
                <ChessBoard chessboard=chessboard />
            </Show>
            <Overlay chessboard=set_chessboard reset=reset chess_board_socket=chess_board_socket />
        </div>
    }
}
