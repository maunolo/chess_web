use crate::components::forms::Form;
use crate::entities::chess_board::signals::ChessBoardSignals;
use crate::entities::chess_board::turns::Turn;
use crate::entities::protocol::{ClientMsg, GameResult, Side};
use crate::i18n::{self, Key};
use leptos::prelude::*;

/// The end of a game, or a checkmate on a free board with validation
#[component]
pub fn CheckMate(chess_board_signals: ChessBoardSignals, show_form: RwSignal<Form>) -> impl IntoView {
    let lang = i18n::current();
    let send = move |msg: ClientMsg| chess_board_signals.send(&msg);

    let winner = move || match chess_board_signals.chess_board().get().turn {
        Turn::Black => lang.side(Side::White),
        Turn::White => lang.side(Side::Black),
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
        (Some(winner), Some(seat)) if winner == seat => lang.t(Key::YouWon),
        (Some(_), Some(_)) => lang.t(Key::YouLost),
        (Some(_), None) => lang.t(Key::GameOver),
        (None, _) => lang.t(Key::Draw),
    };
    let rematch_label = move || {
        let offer = chess_board_signals
            .game()
            .with(|game| game.as_ref().and_then(|game| game.rematch_offer));
        match (offer, chess_board_signals.my_seat()) {
            (Some(offer), Some(seat)) if offer == seat => lang.t(Key::RematchOffered),
            (Some(_), Some(_)) => lang.t(Key::AcceptRematch),
            _ => lang.t(Key::Rematch),
        }
    };

    move || {
        if let Some(result) = result().filter(|_| !dismissed.get()) {
            let seated = chess_board_signals.my_seat().is_some();
            view! {
                <div class="backdrop">
                    <div class="dialog items-stretch">
                        <div class="flex flex-col items-center gap-1">
                            <h1 class="text-2xl font-bold">{title(result)}</h1>
                            <span class="text-sm text-ink-300">{lang.result(result)}</span>
                        </div>
                        {seated
                            .then(|| {
                                view! {
                                    <button class="btn btn-primary" on:click=move |_| send(ClientMsg::Rematch)>
                                        {rematch_label}
                                    </button>
                                }
                            })}
                        <button
                            class="btn btn-secondary"
                            on:click=move |_| {
                                dismissed.set(true);
                                show_form.set(Form::NewGame);
                            }
                        >
                            {lang.t(Key::NewGame)}
                        </button>
                        <button class="btn btn-ghost" on:click=move |_| dismissed.set(true)>
                            {lang.t(Key::Close)}
                        </button>
                    </div>
                </div>
            }
            .into_any()
        } else if chess_board_signals.is_checkmate() {
            view! {
                <div class="backdrop">
                    <div class="dialog items-stretch">
                        <div class="flex flex-col items-center gap-1">
                            <h1 class="text-2xl font-bold">{lang.t(Key::Checkmate)}</h1>
                            <span class="text-sm text-ink-300">{lang.t(Key::TheWinnerIs)} " " {winner}</span>
                        </div>
                        <button class="btn btn-primary" on:click=move |_| send(ClientMsg::Reset)>
                            {lang.t(Key::ResetBoard)}
                        </button>
                        <button class="btn btn-secondary" on:click=move |_| send(ClientMsg::Undo)>
                            {lang.t(Key::UndoLastMove)}
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
