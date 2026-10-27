use std::collections::BTreeMap;

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes},
    ParamSegment, StaticSegment,
};

use crate::components::board::BoardBackground;
use crate::components::chess_board::ChessBoard;

use crate::components::coordinates::Coordinates;
use crate::components::overlay::Overlay;

use crate::components::trash::{Trash, TrashType};
use crate::entities::chess_board::{signals::ChessBoardSignals, ChessBoard as ChessBoardEntity};
use crate::handlers::{pointer_cancel, pointer_move, pointer_up};

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
            // The routes register the pages the server renders. The board stays
            // mounted across them and joins the room of the URL, so moving to
            // another room keeps the same socket.
            <Routes fallback=|| ()>
                <Route path=StaticSegment("") view=|| ()/>
                <Route path=(StaticSegment("g"), ParamSegment("room")) view=|| ()/>
            </Routes>
            <Home/>
        </Router>
    }
}

#[component]
fn Home() -> impl IntoView {
    let fen = "8/8/8/8/8/8/8/8 w - - 0 1";
    let chess_board_entity = ChessBoardEntity::new(fen).unwrap();
    let chess_board_signals = ChessBoardSignals::new(chess_board_entity);
    let should_render = chess_board_signals.should_render();

    view! {
        <div
            class="flex overflow-hidden relative justify-center items-center px-4 w-screen h-screen sm:py-16 sm:px-16 md:py-16 md:px-0"
            on:pointermove=move |e| pointer_move(chess_board_signals, e)
            on:pointerup=move |e| pointer_up(chess_board_signals, e)
            on:pointercancel=move |e| pointer_cancel(chess_board_signals, e)
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
