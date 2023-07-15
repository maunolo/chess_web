use leptos::{SignalGet, SignalGetUntracked, SignalUpdate, SignalWith, SignalWithUntracked};

use crate::entities::chess_board::ChessBoardSignals;
use crate::entities::position::Position;
use crate::utils::class_list::ClassListExt;
use crate::utils::elements::{self, mouse_position_in_bounding};
use crate::utils::events::{EventPositionExt, EventTargetExt};
use crate::utils::style::StyleExt;

use std::fmt;

type Result<T> = std::result::Result<T, EventError>;

#[derive(Debug, Clone)]
pub struct EventError {
    message: String,
}

impl EventError {
    pub fn new(message: &str) -> EventError {
        EventError {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for EventError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EventError: {}", self.message)
    }
}

pub fn interaction_move<E>(chess_board_signals: ChessBoardSignals, event: E)
where
    E: EventPositionExt,
{
    if let Some(piece) = elements::query_selector(".dragging") {
        let position = event.position();
        let client_position = (position.0 as f64, position.1 as f64);

        elements::move_piece(chess_board_signals, &piece, client_position);
    }
}

pub fn interaction_start<E>(chess_board_signals: ChessBoardSignals, event: E)
where
    E: EventPositionExt + EventTargetExt,
{
    let Some(piece) = event.target_element() else {
        log::error!("No target found to start interaction");
        return;
    };
    // select_piece_square(&piece);
    let data_key = piece.get_attribute("data-key").unwrap();
    let data_square = piece.get_attribute("data-square").unwrap();
    let stone_signal;
    if data_square == "deleted" {
        let key = data_key.parse::<usize>().unwrap();
        stone_signal = chess_board_signals
            .stones_signals()
            .with_untracked(|ss| ss.get_deleted_stone(&key));
    } else {
        stone_signal = chess_board_signals
            .stones_signals()
            .with_untracked(|ss| ss.get_board_stone(&data_key));
    }

    let position = event.position();
    // Move the piece to client cursor position
    let client_position = (position.0 as f64, position.1 as f64);

    if let Some(stone_signal) = stone_signal {
        stone_signal.update(|ss| ss.enable_dragging());

        elements::move_piece(chess_board_signals, &piece, client_position);
    }
}

pub fn interaction_end<E>(chess_board_signals: ChessBoardSignals, event: E)
where
    E: EventPositionExt,
{
    if let Some(piece) = elements::query_selector(".dragging") {
        let data_square = piece.get_attribute("data-square").unwrap();
        let data_key = piece.get_attribute("data-key").unwrap();
        let piece_data = piece.get_attribute("data-piece").unwrap();
        let old_pos: String;
        let new_pos: String;

        if data_square == "deleted" {
            let key = data_key.parse::<usize>().unwrap();
            let stone_signal = chess_board_signals
                .stones_signals()
                .with_untracked(|ss| ss.get_deleted_stone(&key));

            if let Some(stone_signal) = stone_signal {
                let stone_signal = stone_signal.get_untracked();
                if stone_signal.position().is_none() {
                } else {
                    old_pos = "deleted".to_string();
                }
            } else {
                old_pos = "deleted".to_string();
            }
        } else {
            old_pos = data_square.to_string();
        }

        if !piece.class_list_include("deleted") {
            let chess_board = piece.parent_element().unwrap();
            let bounding = chess_board.get_bounding_client_rect();
            let position = event.position();

            let client_position = (position.0 as f64, position.1 as f64);
            let (x, y) = mouse_position_in_bounding(client_position, &bounding);
            let is_white_view = chess_board_signals.chess_board().with(|cb| cb.white_view());
            let position = Position::from_ui_position(x, y, is_white_view);

            new_pos = position.to_string();
        } else {
            new_pos = "deleted".to_string();
        }

        if let Some(socket) = chess_board_signals.socket().get().as_ref() {
            let msg = format!("/move {} {} {}", piece_data, old_pos, new_pos);

            match socket.send_with_str(&msg) {
                Ok(_) => {}
                Err(err) => log::error!("error sending message: {:?}", err),
            }
        }

        chess_board_signals.move_piece(piece_data, old_pos, new_pos)
    }
}

// pub fn interaction_end_with_signals<E>(chess_board_signals: ChessBoardSignals, event: E)
// where
//     E: EventPositionExt,
// {
//     if let Ok((piece_data, (old_pos, new_pos))) = interaction_end(chess_board_signals, event) {
//         if let Some(socket) = chess_board_signals.socket().get().as_ref() {
//             let msg = format!("/move {} {} {}", piece_data, old_pos, new_pos);
//
//             match socket.send_with_str(&msg) {
//                 Ok(_) => {}
//                 Err(err) => log::error!("error sending message: {:?}", err),
//             }
//         }
//
//         chess_board_signals.move_piece(piece_data, old_pos, new_pos)
//     };
// }
