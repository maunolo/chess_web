use leptos::prelude::*;

use crate::entities::{
    chess_board::{enums::PromotionKind, signals::ChessBoardSignals},
    protocol::ClientMsg,
};

const CHOICES: [(PromotionKind, char, &str); 4] = [
    (PromotionKind::Queen, 'q', "Queen"),
    (PromotionKind::Rook, 'r', "Rook"),
    (PromotionKind::Bishop, 'b', "Bishop"),
    (PromotionKind::Knight, 'n', "Knight"),
];

/// Asks what a pawn reaching the last rank becomes, then sends its move
#[component]
pub fn PromotionPicker(chess_board_signals: ChessBoardSignals) -> impl IntoView {
    let pick = move |promotion: PromotionKind| {
        let Some(pending) = chess_board_signals.pending_promotion().get_untracked() else {
            return;
        };
        chess_board_signals.pending_promotion().set(None);
        chess_board_signals.send(&ClientMsg::Move {
            piece: pending.piece,
            from: pending.from,
            to: pending.to,
            promotion: Some(promotion),
        });
    };

    move || {
        chess_board_signals.pending_promotion().get().map(|pending| {
            let color = if pending.piece.starts_with('l') { 'l' } else { 'd' };
            view! {
                <div class="z-40 flex absolute w-full h-full justify-center items-center bg-neutral-900/30">
                    <div class="flex flex-col items-center gap-2 bg-white rounded p-3">
                        <span>"Promote to"</span>
                        <div class="flex gap-2">
                            {CHOICES
                                .map(|(promotion, letter, name)| {
                                    view! {
                                        <button
                                            class=format!("promotion-choice {color}{letter}")
                                            aria-label=name
                                            on:click=move |_| pick(promotion)
                                        ></button>
                                    }
                                })}
                        </div>
                    </div>
                </div>
            }
        })
    }
}
