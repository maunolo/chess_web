use leptos::prelude::*;

use crate::{
    components::forms::Form,
    entities::{chess_board::signals::ChessBoardSignals, protocol::ClientMsg},
};

const BANNER_CLASS: &str = "fixed top-16 left-1/2 -translate-x-1/2 z-30 flex items-center gap-3 whitespace-nowrap rounded bg-white/95 px-4 py-2 text-sm shadow";
const BUTTON_CLASS: &str =
    "rounded border border-gray-400 px-2 py-1 hover:border-blue-500 hover:text-blue-500";

/// What a seated player has to act on: an opponent to invite, a draw offer
#[component]
pub fn GameBanner(chess_board_signals: ChessBoardSignals, show_form: RwSignal<Form>) -> impl IntoView {
    let send = move |msg: ClientMsg| chess_board_signals.send(&msg);

    move || {
        let seat = chess_board_signals.my_seat()?;
        let game = chess_board_signals.game().get()?;
        if game.result.is_some() {
            return None;
        }

        if !game.is_full() {
            Some(
                view! {
                    <div class=BANNER_CLASS>
                        <span>"Waiting for an opponent"</span>
                        <button class=BUTTON_CLASS on:click=move |_| show_form.set(Form::Invite)>
                            "Invite"
                        </button>
                    </div>
                }
                .into_any(),
            )
        } else if game.draw_offer == Some(!seat) {
            Some(
                view! {
                    <div class=BANNER_CLASS>
                        <span>"Your opponent offers a draw"</span>
                        <button class=BUTTON_CLASS on:click=move |_| send(ClientMsg::AcceptDraw)>
                            "Accept"
                        </button>
                        <button class=BUTTON_CLASS on:click=move |_| send(ClientMsg::DeclineDraw)>
                            "Decline"
                        </button>
                    </div>
                }
                .into_any(),
            )
        } else {
            None
        }
    }
}
