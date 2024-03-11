use crate::entities::chess_board::signals::ChessBoardSignals;
use crate::entities::chess_board::turns::Turn;
use leptos::*;

#[derive(Clone)]
#[allow(dead_code)]
pub enum Form {
    None,
    Join,
    Username,
    Options,
}

#[component]
pub fn CheckMate(chess_board_signals: ChessBoardSignals) -> impl IntoView {
    let winner = move || match chess_board_signals.chess_board().get().turn {
        Turn::Black => "White",
        Turn::White => "Black",
    };

    let reset = move |_| {
        if let Some(socket) = chess_board_signals.socket().get().as_ref() {
            match socket.send_with_str("/reset") {
                Ok(_) => {}
                Err(err) => log::error!("Error sending message: {:?}", err),
            }
        }
    };

    let undo = move |_| {
        if let Some(socket) = chess_board_signals.socket().get().as_ref() {
            match socket.send_with_str("/undo") {
                Ok(_) => {}
                Err(err) => log::error!("Error sending message: {:?}", err),
            }
        }
    };

    let view = move || {
        if chess_board_signals.is_checkmate() {
            view! {
                <div class="z-40 flex absolute w-full h-full justify-center items-center bg-neutral-900/30">
                    <div class="flex h-fit flex-col justify-center items-center bg-white rounded p-4">
                        <h1 class="text-2xl font-bold mb-2">"Checkmate!"</h1>
                        <span class="text-md font-light">"The winner is " {winner}</span>
                        <button
                            class="border w-full border-gray-400 hover:border-blue-500 hover:text-blue-500 rounded py-2 px-4 my-2"
                            on:click=reset
                        >
                            "Reset Board"
                        </button>
                        <button
                            class="border w-full border-gray-400 hover:border-blue-500 hover:text-blue-500 rounded py-2 px-4"
                            on:click=undo
                        >
                            "Undo Last Move"
                        </button>
                    </div>
                </div>
            }
        } else {
            view! {
                <div class="hidden"></div>
            }
        }
    };

    view
}
