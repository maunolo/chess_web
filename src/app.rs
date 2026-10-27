use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes},
    ParamSegment, StaticSegment,
};

use crate::components::actions::{ActionsLayout, GameActions};
use crate::components::board::BoardBackground;
use crate::components::chess_board::ChessBoard;
use crate::components::coordinates::Coordinates;
use crate::components::forms::Form;
use crate::components::move_list::{MoveStrip, MoveTable};
use crate::components::overlay::Overlay;
use crate::components::player_card::PlayerCard;
use crate::components::room_panel::RoomPanel;
use crate::components::top_bar::TopBar;
use crate::entities::chess_board::{signals::ChessBoardSignals, ChessBoard as ChessBoardEntity};
use crate::entities::protocol::Side;
use crate::handlers::{pointer_cancel, pointer_move, pointer_up};

/// The HTML document the server renders around the app
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover"/>
                <meta name="theme-color" content="#15181c"/>
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
        <Link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=Fira+Code:wght@500;700&display=swap" rel="stylesheet"/>
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
    let white_view = move || chess_board_signals.chess_board().with(|c| c.white_view());

    let show_form = RwSignal::new(Form::None);
    let menu_open = RwSignal::new(false);
    let people_open = RwSignal::new(false);

    view! {
        <div
            class="h-[100dvh] flex flex-col overflow-hidden"
            on:pointermove=move |e| pointer_move(chess_board_signals, e)
            on:pointerup=move |e| pointer_up(chess_board_signals, e)
            on:pointercancel=move |e| pointer_cancel(chess_board_signals, e)
        >
            <TopBar
                chess_board_signals=chess_board_signals
                show_form=show_form
                menu_open=menu_open
                people_open=people_open
            />

            <div class="flex-grow min-h-0 flex flex-col justify-center lg:grid lg:grid-cols-[17.5rem_minmax(0,35rem)_20rem] lg:justify-center lg:items-start lg:gap-5 lg:p-5 lg:overflow-y-auto">
                <RoomPanel chess_board_signals=chess_board_signals/>

                <main class="flex flex-col gap-3 px-3 pb-3 w-full max-w-[calc(100dvh-21rem)] mx-auto lg:max-w-none lg:p-0 lg:mx-0">
                    <PlayerCard chess_board_signals=chess_board_signals side=Side::White/>
                    <PlayerCard chess_board_signals=chess_board_signals side=Side::Black/>
                    <div style="order: 1">
                        <Show
                            when=move || should_render.get()
                            fallback=move || {
                                view! {
                                    <chess-board class="chessboard" id="chessboard">
                                        <BoardBackground/>
                                        <Coordinates white_view=white_view/>
                                    </chess-board>
                                }
                            }
                        >
                            <ChessBoard chess_board_signals=chess_board_signals/>
                        </Show>
                    </div>
                    <div style="order: 3">
                        <MoveStrip chess_board_signals=chess_board_signals/>
                    </div>
                </main>

                <aside class="hidden lg:flex flex-col gap-4">
                    <MoveTable chess_board_signals=chess_board_signals/>
                    <GameActions chess_board_signals=chess_board_signals show_form=show_form layout=ActionsLayout::Panel/>
                </aside>
            </div>

            <GameActions chess_board_signals=chess_board_signals show_form=show_form layout=ActionsLayout::Bar/>

            <Overlay
                chess_board_signals=chess_board_signals
                show_form=show_form
                menu_open=menu_open
                people_open=people_open
            />
        </div>
    }
}
