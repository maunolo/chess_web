use std::collections::BTreeMap;

use leptos::*;

use crate::entities::chess_board::signals::{ChessBoardSignals, StoneSignal};
use crate::entities::chess_board::turns::Turn;
use crate::handlers::interaction_start;

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

#[component]
pub fn Trash<W, T>(
    chess_board_signals: ChessBoardSignals,
    id: TrashType,
    white_view: W,
    trash: T,
) -> impl IntoView
where
    W: Fn() -> bool + 'static,
    T: Fn() -> BTreeMap<usize, RwSignal<StoneSignal>> + Copy + 'static,
{
    let position_css = move || {
        if (matches!(id, TrashType::Dark) && white_view())
            || (matches!(id, TrashType::Light) && !white_view())
        {
            "-bottom-10 sm:-bottom-14 rounded-b".to_string()
        } else {
            "-top-10 sm:-top-14 rounded-t".to_string()
        }
    };

    let trash_signals = move || {
        trash()
            .into_iter()
            .filter(move |(_, stone_signal)| {
                stone_signal.get().stone().color().to_string() == String::from(id)
            })
            .collect::<Vec<(usize, RwSignal<StoneSignal>)>>()
    };
    let active = move || match (chess_board_signals.chess_board().get().turn, id) {
        (Turn::Black, TrashType::Light) => "border-2 border-neutral-200".to_string(),
        (Turn::White, TrashType::Dark) => "border-2 border-neutral-200".to_string(),
        _ => "p-[2px]".to_string(),
    };

    let trash_class = move || format!("trash {} {}", position_css(), active());

    let trash_id = move || format!("{}-trash", String::from(id));

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
                on:mousedown=move |e| interaction_start(chess_board_signals, e)
                on:touchstart=move |e| interaction_start(chess_board_signals, e)
                on:dragstart=move |e| e.prevent_default()
                data-square="deleted"
                data-piece=stone().image_class()
                data-key=move || idx.to_string()
                data-deleted=move || format!("{}", stone_signal.get().is_deleted())
            ></div>
        }
    };

    view! {
        <div class=trash_class data-trash=move || String::from(id) id=trash_id>
            <For
                each=trash_signals
                key=move |(key, _)| key.to_string()
                children=piece_view
            />
        </div>
    }
}
