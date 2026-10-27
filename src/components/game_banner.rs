use leptos::prelude::*;

use crate::{
    components::forms::Form,
    entities::{chess_board::signals::ChessBoardSignals, protocol::ClientMsg},
};

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
                    <div class="banner">
                        <span>"Waiting for an opponent"</span>
                        <button class="btn btn-primary h-9" on:click=move |_| show_form.set(Form::Invite)>
                            "Invite"
                        </button>
                    </div>
                }
                .into_any(),
            )
        } else if game.draw_offer == Some(!seat) {
            Some(
                view! {
                    <div class="banner">
                        <span>"Your opponent offers a draw"</span>
                        <button class="btn btn-primary h-9" on:click=move |_| send(ClientMsg::AcceptDraw)>
                            "Accept"
                        </button>
                        <button class="btn btn-secondary h-9" on:click=move |_| send(ClientMsg::DeclineDraw)>
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
