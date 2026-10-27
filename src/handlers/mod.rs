use std::cell::RefCell;

use leptos::prelude::*;
use web_sys::{Element, PointerEvent};

use crate::entities::chess_board::signals::{ChessBoardSignals, PendingMove, Selection, StoneSignal};
use crate::entities::position::{Position, DELETED};
use crate::entities::protocol::ClientMsg;
use crate::utils::class_list::ClassListExt;
use crate::utils::elements::{self, mouse_in_bounding, mouse_position_in_bounding, query_selector};
use crate::utils::events::{EventPositionExt, EventTargetExt};
use crate::utils::style::StyleExt;

/// Moving the pointer this far turns a press on a piece into a drag
const DRAG_THRESHOLD_PX: f64 = 6.0;

/// A pointer pressed on a piece, not yet known to be a tap or a drag
struct Press {
    element: Element,
    piece: String,
    key: String,
    square: String,
    start: (f64, f64),
    /// Opponent pieces can't be dragged, but tapping one captures it
    draggable: bool,
    dragging: bool,
}

thread_local! {
    static PRESS: RefCell<Option<Press>> = const { RefCell::new(None) };
}

fn client_position<E: EventPositionExt>(event: &E) -> (f64, f64) {
    let (x, y) = event.position();
    (x as f64, y as f64)
}

fn square_name(position: &Option<Position>) -> String {
    position
        .as_ref()
        .map(|position| position.to_string())
        .unwrap_or(DELETED.to_string())
}

/// Pressing a piece on the board or in the trash
pub fn pointer_down(chess_board_signals: ChessBoardSignals, event: PointerEvent) {
    if !event.is_primary() || event.button() != 0 {
        return;
    }
    let Some(element) = event.target_element() else {
        log::error!("No target found to start interaction");
        return;
    };
    let attribute = |name: &str| element.get_attribute(name).unwrap_or_default();
    let (piece, key, square) = (attribute("data-piece"), attribute("data-key"), attribute("data-square"));

    // keep getting the pointer's events while it moves off the piece
    let _ = element.set_pointer_capture(event.pointer_id());
    event.prevent_default();

    let draggable = chess_board_signals.can_move(&piece, square == DELETED);
    PRESS.set(Some(Press {
        element,
        piece,
        key,
        square,
        start: client_position(&event),
        draggable,
        dragging: false,
    }));
}

pub fn pointer_move(chess_board_signals: ChessBoardSignals, event: PointerEvent) {
    let position = client_position(&event);

    PRESS.with_borrow_mut(|press| {
        let Some(press) = press else {
            return;
        };

        if !press.dragging {
            let distance = (position.0 - press.start.0).hypot(position.1 - press.start.1);
            if !press.draggable || distance < DRAG_THRESHOLD_PX {
                return;
            }

            press.dragging = true;
            let stone_signal =
                get_stone_signal(chess_board_signals, press.key.clone(), press.square.clone());
            stone_signal.update(|ss| ss.enable_dragging());
            chess_board_signals.selected().set(Some(Selection {
                piece: press.piece.clone(),
                from: Position::parse_square(&press.square).ok().flatten(),
            }));
        }

        elements::move_piece(&press.element, position);
    });
}

pub fn pointer_up(chess_board_signals: ChessBoardSignals, event: PointerEvent) {
    match PRESS.take() {
        Some(press) if press.dragging => {
            chess_board_signals.selected().set(None);
            finish_drag(chess_board_signals, event);
        }
        Some(press) => tap_piece(chess_board_signals, press),
        None => tap_board(chess_board_signals, event),
    }
}

/// The browser took the pointer away, e.g. for a system gesture
pub fn pointer_cancel(chess_board_signals: ChessBoardSignals, event: PointerEvent) {
    if PRESS.with_borrow(|press| press.as_ref().is_some_and(|press| press.dragging)) {
        pointer_up(chess_board_signals, event);
    } else {
        PRESS.set(None);
    }
}

fn same_color(piece: &str, other: &str) -> bool {
    piece.chars().next() == other.chars().next()
}

fn tap_piece(chess_board_signals: ChessBoardSignals, press: Press) {
    let tapped = Selection {
        piece: press.piece,
        from: Position::parse_square(&press.square).ok().flatten(),
    };

    match chess_board_signals.selected().get_untracked() {
        Some(selected) if selected == tapped => chess_board_signals.selected().set(None),
        // a piece of the other color: capture it
        Some(selected) if !same_color(&selected.piece, &tapped.piece) && tapped.from.is_some() => {
            commit_move(chess_board_signals, selected, tapped.from)
        }
        _ if press.draggable => chess_board_signals.selected().set(Some(tapped)),
        _ => chess_board_signals.selected().set(None),
    }
}

/// A tap that didn't start on a piece: moves the selected piece there
fn tap_board(chess_board_signals: ChessBoardSignals, event: PointerEvent) {
    let Some(selected) = chess_board_signals.selected().get_untracked() else {
        return;
    };
    let Some(target) = event.target_element() else {
        return;
    };
    let closest = |selector: &str| target.closest(selector).ok().flatten();

    if closest("[data-trash]").is_some() {
        if !chess_board_signals.is_game() {
            commit_move(chess_board_signals, selected, None);
        }
    } else if let Some(board) = closest("#chessboard") {
        let position = client_position(&event);
        if !mouse_in_bounding(position, &board.get_bounding_client_rect()) {
            return;
        }
        let to = get_piece_position(chess_board_signals, event);

        if selected.from.as_ref() == Some(&to) {
            chess_board_signals.selected().set(None);
        } else {
            commit_move(chess_board_signals, selected, Some(to));
        }
    }
}

/// Plays a tapped move: shows it right away, then sends it
fn commit_move(chess_board_signals: ChessBoardSignals, selected: Selection, to: Option<Position>) {
    chess_board_signals.selected().set(None);
    if selected.from == to {
        return;
    }

    chess_board_signals.move_piece(
        selected.piece.clone(),
        square_name(&selected.from),
        square_name(&to),
    );
    send_move(chess_board_signals, selected.piece, selected.from, to);
}

/// Sends a move the board already shows, asking first what a promoting pawn
/// becomes when the rules are enforced
fn send_move(
    chess_board_signals: ChessBoardSignals,
    piece: String,
    from: Option<Position>,
    to: Option<Position>,
) {
    chess_board_signals
        .last_move()
        .set(Some((from.clone(), to.clone())));

    let last_rank = match piece.as_str() {
        "lp" => Some(0),
        "dp" => Some(7),
        _ => None,
    };
    let promotes = chess_board_signals.validation()
        && from.is_some()
        && last_rank.is_some()
        && to.as_ref().map(|to| to.y) == last_rank;

    let (from, to) = (square_name(&from), square_name(&to));
    if promotes {
        chess_board_signals
            .pending_promotion()
            .set(Some(PendingMove { piece, from, to }));
    } else {
        chess_board_signals.send(&ClientMsg::Move {
            piece,
            from,
            to,
            promotion: None,
        });
    }
}

/// Drops a dragged piece where the pointer is: on a square or in the trash
fn finish_drag<E>(chess_board_signals: ChessBoardSignals, event: E)
where
    E: EventPositionExt,
{
    let dark_trash = query_selector("[data-trash=\"dark\"]").unwrap();
    let light_trash = query_selector("[data-trash=\"light\"]").unwrap();
    let chess_board = query_selector("#chessboard").unwrap();
    dark_trash.class_list_remove("dragging-over");
    light_trash.class_list_remove("dragging-over");
    chess_board.class_list_remove("dragging-over");

    if let Some(piece) = elements::query_selector(".dragging") {
        elements::delete_deleted_piece_clone(&piece);
        elements::delete_restored_piece_clone(&piece);

        let data_key = piece.get_attribute("data-key").unwrap();
        let data_deleted = piece
            .get_attribute("data-deleted")
            .unwrap()
            .parse::<bool>()
            .unwrap();
        let piece_data = piece.get_attribute("data-piece").unwrap();
        let data_square = piece.get_attribute("data-square").unwrap();
        let old_pos: String;
        let new_pos: String;

        let stone_signal =
            get_stone_signal(chess_board_signals, data_key.clone(), data_square.clone());

        stone_signal.update(|ss| {
            ss.disable_dragging();
        });

        if data_deleted && (data_square != "deleted") {
            stone_signal.update(|stone| {
                stone.delete();
            });

            chess_board_signals.stones_signals().update(|ss| {
                if let Some(stone) = ss.remove_board_stone(data_key.clone()) {
                    ss.add_deleted_stone_signal(stone);
                };
            });
        } else if !data_deleted && (data_square == "deleted") {
            stone_signal.update(|stone| {
                stone.restore();
            });

            let new_key = stone_signal.with_untracked(|ss| ss.unique_key());

            chess_board_signals.stones_signals().update(|ss| {
                let key = data_key.parse::<usize>().expect("Not able to parse key");
                if let Some(stone) = ss.remove_deleted_stone(key) {
                    ss.add_board_stone_signal(new_key, stone);
                };
            });
        }

        piece.remove_style("transition");
        piece.remove_style("transform");

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
            let new_position = get_piece_position(chess_board_signals, event);
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

        if old_pos == new_pos {
            return;
        }

        let (Ok(old_pos), Ok(new_pos)) = (
            Position::parse_square(&old_pos),
            Position::parse_square(&new_pos),
        ) else {
            return;
        };

        chess_board_signals.chess_board().update(|chessboard| {
            let _ = chessboard.move_piece(&piece_data, old_pos.clone(), new_pos.clone());
        });
        send_move(chess_board_signals, piece_data, old_pos, new_pos);
    }
}

pub fn get_stone_signal(
    chess_board_signals: ChessBoardSignals,
    key: String,
    data_square: String,
) -> RwSignal<StoneSignal> {
    if data_square == "deleted" {
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

pub fn get_piece_position<E>(chess_board_signals: ChessBoardSignals, event: E) -> Position
where
    E: EventPositionExt,
{
    let chess_board = query_selector("#chessboard").unwrap();
    let bounding = chess_board.get_bounding_client_rect();

    let (x, y) = mouse_position_in_bounding(client_position(&event), &bounding);
    let is_white_view = chess_board_signals
        .chess_board()
        .with_untracked(|cb| cb.white_view());
    Position::from_ui_position(x, y, is_white_view)
}
