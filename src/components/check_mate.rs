use crate::components::forms::Form;
use crate::entities::chess_board::signals::ChessBoardSignals;
use crate::entities::chess_board::turns::Turn;
use crate::entities::protocol::{ClientMsg, GameResult};
use leptos::prelude::*;

const OVERLAY_CLASS: &str =
    "z-40 flex absolute w-full h-full justify-center items-center bg-neutral-900/30";
const CARD_CLASS: &str = "flex h-fit flex-col justify-center items-center bg-white rounded p-4";
const BUTTON_CLASS: &str =
    "border w-full border-gray-400 hover:border-blue-500 hover:text-blue-500 rounded py-2 px-4 my-1";

/// The end of a game, or a checkmate on a free board with validation
#[component]
pub fn CheckMate(chess_board_signals: ChessBoardSignals, show_form: RwSignal<Form>) -> impl IntoView {
    let send = move |msg: ClientMsg| chess_board_signals.send(&msg);

    let winner = move || match chess_board_signals.chess_board().get().turn {
        Turn::Black => "White",
        Turn::White => "Black",
    };

    let result = move || {
        chess_board_signals
            .game()
            .with(|game| game.as_ref().and_then(|game| game.result))
    };
    // closing keeps the final position in view, a rematch shows the next result
    let dismissed = RwSignal::new(false);
    Effect::new(move |_| {
        if result().is_none() {
            dismissed.set(false);
        }
    });

    let title = move |result: GameResult| match (result.winner(), chess_board_signals.my_seat()) {
        (Some(winner), Some(seat)) if winner == seat => "You won!",
        (Some(_), Some(_)) => "You lost",
        (Some(_), None) => "Game over",
        (None, _) => "Draw",
    };
    let rematch_label = move || {
        let offer = chess_board_signals
            .game()
            .with(|game| game.as_ref().and_then(|game| game.rematch_offer));
        match (offer, chess_board_signals.my_seat()) {
            (Some(offer), Some(seat)) if offer == seat => "Rematch offered",
            (Some(_), Some(_)) => "Accept rematch",
            _ => "Rematch",
        }
    };

    move || {
        if let Some(result) = result().filter(|_| !dismissed.get()) {
            let seated = chess_board_signals.my_seat().is_some();
            view! {
                <div class=OVERLAY_CLASS>
                    <div class=CARD_CLASS>
                        <h1 class="text-2xl font-bold mb-2">{title(result)}</h1>
                        <span class="text-md font-light mb-2">{result.description()}</span>
                        {seated
                            .then(|| {
                                view! {
                                    <button class=BUTTON_CLASS on:click=move |_| send(ClientMsg::Rematch)>
                                        {rematch_label}
                                    </button>
                                }
                            })}
                        <button
                            class=BUTTON_CLASS
                            on:click=move |_| {
                                dismissed.set(true);
                                show_form.set(Form::NewGame);
                            }
                        >
                            "New game"
                        </button>
                        <button class=BUTTON_CLASS on:click=move |_| dismissed.set(true)>
                            "Close"
                        </button>
                    </div>
                </div>
            }
            .into_any()
        } else if chess_board_signals.is_checkmate() {
            view! {
                <div class=OVERLAY_CLASS>
                    <div class=CARD_CLASS>
                        <h1 class="text-2xl font-bold mb-2">"Checkmate!"</h1>
                        <span class="text-md font-light">"The winner is " {winner}</span>
                        <button class=BUTTON_CLASS on:click=move |_| send(ClientMsg::Reset)>
                            "Reset Board"
                        </button>
                        <button class=BUTTON_CLASS on:click=move |_| send(ClientMsg::Undo)>
                            "Undo Last Move"
                        </button>
                    </div>
                </div>
            }
            .into_any()
        } else {
            view! { <div class="hidden"></div> }.into_any()
        }
    }
}
