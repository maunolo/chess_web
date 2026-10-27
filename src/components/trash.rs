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

/// The captured pieces of one color. On a free board a piece dropped here
/// leaves the board, and one dragged out goes back.
#[component]
pub fn Trash<T>(chess_board_signals: ChessBoardSignals, id: TrashType, trash: T) -> impl IntoView
where
    T: Fn() -> BTreeMap<usize, RwSignal<StoneSignal>> + Copy + Send + 'static,
{
    let trash_signals = move || {
        trash()
            .into_iter()
            .filter(move |(_, stone_signal)| {
                stone_signal.get().stone().color().to_string() == String::from(id)
            })
            .collect::<Vec<(usize, RwSignal<StoneSignal>)>>()
    };

    let piece_view = move |(idx, stone_signal): (usize, RwSignal<StoneSignal>)| {
        let stone = move || stone_signal.get().stone();
        let dragging_class = move || {
            if stone_signal.get().is_dragging() {
                "dragging".to_string()
            } else {
                "".to_string()
            }
        };

        view! {
            <div
                class=move || format!("piece {} deleted {}", stone().image_class(), dragging_class())
                on:pointerdown=move |e| pointer_down(chess_board_signals, e)
                on:dragstart=move |e| e.prevent_default()
                data-square="deleted"
                data-piece=stone().image_class()
                data-key=move || idx.to_string()
                data-deleted=move || format!("{}", stone_signal.get().is_deleted())
            ></div>
        }
    };

    view! {
        <div class="trash" data-trash=String::from(id) id=format!("{}-trash", String::from(id))>
            <For
                each=trash_signals
                key=move |(key, _)| key.to_string()
                children=piece_view
            />
        </div>
    }
}
