use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::components::chess_board::ChessBoard;

use crate::components::overlay::Overlay;

use crate::entities::chess_board::{
    ChessBoard as ChessBoardEntity, ChessBoardSignalsBuilder, StonesSignals,
};
use crate::entities::room::RoomStatus;
use crate::handlers::{interaction_end, interaction_move};
use crate::utils::set_touch_events;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    provide_meta_context(cx);

    let app_title = env!("APP_TITLE");

    view! { cx,
        <Stylesheet id="leptos" href="/style.css"/>
        <Title text=app_title/>
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
    let fen = "8/8/8/8/8/8/8/8 w - - 0 1";
    let chess_board_entity = ChessBoardEntity::new(fen).unwrap();
    let chess_board = create_rw_signal::<ChessBoardEntity>(cx, chess_board_entity);
    let chess_board_socket = create_rw_signal::<Option<web_sys::WebSocket>>(cx, None);
    let room_status = create_rw_signal::<Option<RoomStatus>>(cx, None);
    let stones_signals = create_rw_signal::<StonesSignals>(cx, StonesSignals::new());

    let chess_board_signals = ChessBoardSignalsBuilder::new()
        .cx(cx)
        .chess_board(chess_board)
        .room_status(room_status)
        .chess_board_socket(chess_board_socket)
        .stones_signals(stones_signals)
        .build()
        .unwrap();

    set_touch_events(chess_board_signals);

    view! { cx,
        <div
            class="flex overflow-hidden relative justify-center items-center px-4 w-screen h-screen sm:py-16 sm:px-16 md:py-16 md:px-0"
            on:mousemove=move |e| interaction_move(chess_board_signals, e)
            on:mouseup=move |e| interaction_end(chess_board_signals, e)
        >
            <ChessBoard chess_board_signals=chess_board_signals/>
            <Overlay chess_board_signals=chess_board_signals/>
        </div>
    }
}
