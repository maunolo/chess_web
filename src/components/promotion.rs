use leptos::prelude::*;

use crate::{
    entities::{
        chess_board::{enums::PromotionKind, signals::ChessBoardSignals},
        protocol::ClientMsg,
    },
    i18n::{self, Key},
};

const CHOICES: [(PromotionKind, char, Key); 4] = [
    (PromotionKind::Queen, 'q', Key::Queen),
    (PromotionKind::Rook, 'r', Key::Rook),
    (PromotionKind::Bishop, 'b', Key::Bishop),
    (PromotionKind::Knight, 'n', Key::Knight),
];

/// Asks what a pawn reaching the last rank becomes, then sends its move
#[component]
pub fn PromotionPicker(chess_board_signals: ChessBoardSignals) -> impl IntoView {
    let lang = i18n::current();
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
                <div class="backdrop">
                    <div class="dialog w-auto items-center">
                        <span class="dialog-title">{lang.t(Key::PromoteTo)}</span>
                        <div class="flex gap-2">
                            {CHOICES
                                .map(|(promotion, letter, name)| {
                                    view! {
                                        <button
                                            class=format!("promotion-choice {color}{letter}")
                                            aria-label=lang.t(name)
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
