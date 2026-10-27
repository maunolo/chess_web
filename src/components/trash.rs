use std::collections::BTreeMap;

use leptos::prelude::*;

use crate::entities::chess_board::signals::{ChessBoardSignals, StoneSignal};
use crate::handlers::pointer_down;

#[derive(Clone, Copy, PartialEq)]
pub enum TrashType {
    Dark,
    Light,
}

impl From<TrashType> for String {
    fn from(trash_type: TrashType) -> Self {
        match trash_type {
            TrashType::Dark => "dark".to_string(),
            TrashType::Light => "light".to_string(),
        }
    }
}

/// Captured pieces are shown by kind, the most valuable first
const KIND_ORDER: &str = "qrbnpk";

/// The captured pieces of one color, identical ones collapsed into one piece
/// with a count. On a free board a piece dropped here leaves the board, and
/// one dragged out goes back.
#[component]
pub fn Trash<T>(chess_board_signals: ChessBoardSignals, id: TrashType, trash: T) -> impl IntoView
where
    T: Fn() -> BTreeMap<usize, RwSignal<StoneSignal>> + Copy + Send + 'static,
{
    // `(piece, count, key and signal of one of them)`, the one a drag picks up
    let groups = move || {
        let mut groups: Vec<(String, usize, usize, RwSignal<StoneSignal>)> = Vec::new();
        for (idx, stone_signal) in trash() {
            let stone = stone_signal.get().stone();
            if stone.color().to_string() != String::from(id) {
                continue;
            }
            let piece = stone.image_class();
            match groups.iter_mut().find(|group| group.0 == piece) {
                Some(group) => group.1 += 1,
                None => groups.push((piece, 1, idx, stone_signal)),
            }
        }
        groups.sort_by_key(|(piece, ..)| {
            piece
                .chars()
                .nth(1)
                .and_then(|kind| KIND_ORDER.find(kind))
                .unwrap_or(KIND_ORDER.len())
        });
        groups
    };

    let piece_view = move |(piece, count, idx, stone_signal): (String, usize, usize, RwSignal<StoneSignal>)| {
        let dragging_class = move || {
            if stone_signal.get().is_dragging() {
                "dragging".to_string()
            } else {
                "".to_string()
            }
        };
        let class = {
            let piece = piece.clone();
            move || format!("piece {} deleted {}", piece, dragging_class())
        };

        view! {
            <div
                class=class
                on:pointerdown=move |e| pointer_down(chess_board_signals, e)
                on:dragstart=move |e| e.prevent_default()
                data-square="deleted"
                data-piece=piece
                data-key=idx.to_string()
                data-deleted=move || format!("{}", stone_signal.get().is_deleted())
            >
                {(count > 1).then(|| view! { <span class="count">{format!("{count}x")}</span> })}
            </div>
        }
    };

    view! {
        <div class="trash" data-trash=String::from(id) id=format!("{}-trash", String::from(id))>
            // the count and the picked key are part of the key, so a group
            // re-renders when a piece joins or leaves it
            <For
                each=groups
                key=|(piece, count, idx, _)| (piece.clone(), *count, *idx)
                children=piece_view
            />
        </div>
    }
}
