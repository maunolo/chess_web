use std::collections::BTreeMap;

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

use crate::components::board::BoardBackground;
use crate::components::chess_board::ChessBoard;

use crate::components::coordinates::Coordinates;
use crate::components::overlay::Overlay;

use crate::components::trash::{Trash, TrashType};
use crate::entities::chess_board::{
    signals::{ChessBoardSignalsBuilder, StonesSignals},
    ChessBoard as ChessBoardEntity,
};
use crate::entities::notification::{Notification, NotifyType};
use crate::entities::room::RoomStatus;
use crate::handlers::{interaction_end, interaction_move};

/// The HTML document the server renders around the app
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let app_title = option_env!("APP_TITLE").unwrap_or("Chess");

    view! {
        <Stylesheet id="leptos" href="/pkg/chess_web.css"/>
        <Title text=app_title/>
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
        <Link rel="preconnect" href="https://fonts.googleapis.com"/>
        <Link rel="preconnect" href="https://fonts.gstatic.com" crossorigin=""/>
        <Link href="https://fonts.googleapis.com/css2?family=Fira+Code:wght@300;400;500;600;700&display=swap" rel="stylesheet"/>
        <Router>
            <Routes fallback=|| "Page not found.">
                <Route path=StaticSegment("") view=Home/>
            </Routes>
        </Router>
    }
}

#[component]
fn Home() -> impl IntoView {
    let fen = "8/8/8/8/8/8/8/8 w - - 0 1";
    let chess_board_entity = ChessBoardEntity::new(fen).unwrap();
    let should_render = RwSignal::new(false);
    let chess_board = RwSignal::new(chess_board_entity);
    // A `WebSocket` is a JS object, so its signal stays on the thread that made it
    let chess_board_socket = RwSignal::new_local(None::<web_sys::WebSocket>);
    let room_status = RwSignal::new(None::<RoomStatus>);
    let stones_signals = RwSignal::new(StonesSignals::new());
    let notification = RwSignal::new(Notification::new("".to_string(), NotifyType::Success));

    let chess_board_signals = ChessBoardSignalsBuilder::new()
        .chess_board(chess_board)
        .room_status(room_status)
        .chess_board_socket(chess_board_socket)
        .stones_signals(stones_signals)
        .should_render(should_render)
        .notification(notification)
        .build()
        .unwrap();

    view! {
        <div
            class="flex overflow-hidden relative justify-center items-center px-4 w-screen h-screen sm:py-16 sm:px-16 md:py-16 md:px-0"
            on:touchmove=move |e| interaction_move(e)
            on:touchend=move |e| interaction_end(chess_board_signals, e)
            on:mousemove=move |e| interaction_move(e)
            on:mouseup=move |e| interaction_end(chess_board_signals, e)
        >
            <Show
                when=move || should_render.get()
                fallback=move || {
                    view! {
                        <chess-board class="chessboard" id="chessboard">
                            <BoardBackground/>
                            <Coordinates white_view=move || true/>
                            <Trash id=TrashType::Dark chess_board_signals=chess_board_signals white_view=move || true trash=move || BTreeMap::new()/>
                            <Trash id=TrashType::Light chess_board_signals=chess_board_signals white_view=move || true trash=move || BTreeMap::new()/>
                        </chess-board>
                    }
                }
            >
                <ChessBoard chess_board_signals=chess_board_signals/>
            </Show>
            <Overlay chess_board_signals=chess_board_signals/>
        </div>
    }
}
