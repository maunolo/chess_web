use leptos::{RwSignal, SignalGetUntracked, SignalUpdate, SignalWithUntracked};
use web_sys::Element;

use crate::entities::chess_board::{ChessBoardSignals, StoneSignal};
use crate::entities::position::Position;
use crate::utils::elements::{self, mouse_position_in_bounding};
use crate::utils::events::{EventPositionExt, EventTargetExt};
use crate::utils::style::StyleExt;

pub fn interaction_move<E>(chess_board_signals: ChessBoardSignals, event: E)
where
    E: EventPositionExt,
{
    if let Some(piece) = elements::query_selector(".dragging") {
        let data_key = piece.get_attribute("data-key").unwrap();
        let data_deleted = piece.get_attribute("data-deleted").unwrap();

        let stone_signal = get_stone_signal(chess_board_signals, data_key, data_deleted);

        let position = event.position();
        let client_position = (position.0 as f64, position.1 as f64);

        elements::move_piece(chess_board_signals, stone_signal, &piece, client_position);
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
    let data_deleted = piece.get_attribute("data-deleted").unwrap();

    let stone_signal = get_stone_signal(chess_board_signals, data_key, data_deleted);

    let position = event.position();
    // Move the piece to client cursor position
    let client_position = (position.0 as f64, position.1 as f64);

    stone_signal.update(|ss| ss.enable_dragging());

    elements::move_piece(chess_board_signals, stone_signal, &piece, client_position);
}

pub fn interaction_end<E>(chess_board_signals: ChessBoardSignals, event: E)
where
    E: EventPositionExt,
{
    if let Some(piece) = elements::query_selector(".dragging") {
        let data_key = piece.get_attribute("data-key").unwrap();
        let data_deleted = piece.get_attribute("data-deleted").unwrap();
        let piece_data = piece.get_attribute("data-piece").unwrap();
        let old_pos: String;
        let new_pos: String;

        let stone_signal = get_stone_signal(chess_board_signals, data_key, data_deleted);

        piece.remove_style("transition");
        piece.remove_style("transform");
        stone_signal.update(|ss| {
            ss.disable_dragging();
            ss.set_transform(None);
        });

        old_pos = stone_signal
            .with_untracked(|ss| ss.position())
            .map(|p| p.to_string())
            .unwrap_or("deleted".to_string());

        if stone_signal.with_untracked(|ss| ss.is_deleted()) {
            stone_signal.update(|ss| {
                ss.set_position(None);
                ss.delete();
            });
            new_pos = "deleted".to_string();
        } else {
            let tmp_key = stone_signal.with_untracked(|ss| ss.unique_key());
            let new_position = get_piece_position(chess_board_signals, &piece, event);
            if stone_signal.with_untracked(|ss| ss.position()) != Some(new_position.clone()) {
                stone_signal.update(|ss| ss.set_position(Some(new_position.clone())));
            }
            let new_key = stone_signal.with_untracked(|ss| ss.unique_key());
            if tmp_key != new_key {
                if let Some(new_pos_stone) = chess_board_signals
                    .chess_board()
                    .with_untracked(|cb| cb.stone_at(new_position.x(), new_position.y()).cloned())
                {
                    let stone_key =
                        StoneSignal::new(Some(new_position.clone()), new_pos_stone).unique_key();

                    if let Some(stone_signal) = chess_board_signals
                        .stones_signals()
                        .with_untracked(|ss| ss.get_board_stone(&stone_key))
                    {
                        stone_signal.update(|ss| {
                            ss.set_position(None);
                            ss.delete();
                        });
                    };

                    chess_board_signals.stones_signals().update(|stones| {
                        if let Some(stone) = stones.remove_board_stone(stone_key) {
                            stones.add_deleted_stone_signal(stone);
                        };
                    });
                }
                chess_board_signals.stones_signals().update(|ss| {
                    if let Some(stone) = ss.remove_board_stone(tmp_key) {
                        ss.add_board_stone_signal(new_key, stone);
                    }
                });
            }

            new_pos = new_position.to_string();
        }

        log::debug!("old_pos: {}, new_pos: {}", old_pos, new_pos);

        if old_pos == new_pos {
            return;
        }

        if let Some(socket) = chess_board_signals.socket().get_untracked().as_ref() {
            let msg = format!("/move {} {} {}", piece_data, old_pos, new_pos);

            match socket.send_with_str(&msg) {
                Ok(_) => {}
                Err(err) => log::error!("error sending message: {:?}", err),
            }
        }

        let old_pos = match old_pos.as_str() {
            "deleted" => None,
            _ => Some(old_pos.parse::<Position>().unwrap()),
        };
        let new_pos = match new_pos.as_str() {
            "deleted" => None,
            _ => Some(new_pos.parse::<Position>().unwrap()),
        };

        chess_board_signals.chess_board().update(|chessboard| {
            chessboard.move_piece(&piece_data, old_pos, new_pos);
        });
    }
}

pub fn get_stone_signal(
    chess_board_signals: ChessBoardSignals,
    key: String,
    data_deleted: String,
) -> RwSignal<StoneSignal> {
    if data_deleted == "true" {
        let key = key.parse::<usize>().unwrap();
        chess_board_signals
            .stones_signals()
            .with_untracked(|ss| ss.get_deleted_stone(&key))
            .expect("stone_signal should be deleted")
    } else {
        chess_board_signals
            .stones_signals()
            .with_untracked(|ss| ss.get_board_stone(&key))
            .expect("stone_signal should be in the board")
    }
}

pub fn get_piece_position<E>(
    chess_board_signals: ChessBoardSignals,
    piece: &Element,
    event: E,
) -> Position
where
    E: EventPositionExt,
{
    let chess_board = piece.parent_element().unwrap();
    let bounding = chess_board.get_bounding_client_rect();
    let position = event.position();

    let client_position = (position.0 as f64, position.1 as f64);
    let (x, y) = mouse_position_in_bounding(client_position, &bounding);
    let is_white_view = chess_board_signals
        .chess_board()
        .with_untracked(|cb| cb.white_view());
    Position::from_ui_position(x, y, is_white_view)
}
