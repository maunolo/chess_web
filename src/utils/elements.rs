use leptos::{SignalGetUntracked, SignalUpdate, SignalUpdateUntracked, SignalWithUntracked};
use web_sys::{Document, DomRect, Element};

use crate::entities::chess_board::{ChessBoardSignals, Transform};

use super::class_list::ClassListExt;
use super::style::StyleExt;

pub fn document() -> Document {
    web_sys::window().unwrap().document().unwrap()
}

pub fn query_selector(class: &str) -> Option<Element> {
    document().query_selector(class).unwrap()
}

pub fn move_piece(
    chess_board_signals: ChessBoardSignals,
    piece: &Element,
    client_position: (f64, f64),
) {
    let parent = piece.parent_element().unwrap();
    let data_key = piece.get_attribute("data-key").unwrap();
    let data_square = piece.get_attribute("data-square").unwrap();
    let dark_trash = query_selector("[data-trash=\"dark\"]").unwrap();
    let light_trash = query_selector("[data-trash=\"light\"]").unwrap();
    let in_dark_trash = mouse_in_bounding(client_position, &dark_trash.get_bounding_client_rect());
    let in_light_trash =
        mouse_in_bounding(client_position, &light_trash.get_bounding_client_rect());

    let stone_signal;
    if data_square == "deleted" {
        let key = data_key.parse::<usize>().expect("Not able to parse key");
        log::debug!(
            "deleted stones: {:?}",
            chess_board_signals
                .stones_signals()
                .get_untracked()
                .deleted_stones()
        );
        stone_signal = chess_board_signals
            .stones_signals()
            .with_untracked(|ss| ss.get_deleted_stone(&key));

        if !(in_dark_trash || in_light_trash) {
            log::debug!("Restoring stone {}", key);
            if let Some(stone_signal) = stone_signal {
                let clone = stone_signal.get_untracked();
                let position = clone.position();
            }
            chess_board_signals.stones_signals().update(|ss| {
                if let Some(stone) = ss.remove_deleted_stone(key) {
                    let key = stone().unique_key();
                    ss.add_board_stone_signal(key, stone);
                };
            });
        }
    } else {
        stone_signal = chess_board_signals
            .stones_signals()
            .with_untracked(|ss| ss.get_board_stone(&data_key));

        if in_dark_trash || in_light_trash {
            chess_board_signals.stones_signals().update(|ss| {
                if let Some(stone) = ss.remove_board_stone(data_key.clone()) {
                    ss.add_deleted_stone_signal(stone);
                };
            });

            if let Some(stone_signal) = stone_signal {
                // let stone = stone_signal.get_untracked();
                // let old_key = stone.old_key();
                // let position = stone.position();
                //
                stone_signal.update(|stone| {
                    stone.set_position(None);

                    stone.set_transform(None);
                });
            }
        }
    }

    if let Some(stone_signal) = stone_signal {
        if stone_signal.with_untracked(|ss| ss.position().is_some()) {
            piece.set_style("transition", "none");
            piece.set_style("transform", &translate_value(client_position, &parent));
        }
    }
}

pub fn soft_delete_piece(piece: &Element) {
    let parent = piece.parent_element().unwrap();
    let trash = if is_piece_dark_variant(&piece) {
        query_selector("[data-trash=\"dark\"]").unwrap()
    } else {
        query_selector("[data-trash=\"light\"]").unwrap()
    };

    let data_piece = piece.get_attribute("data-piece").unwrap();
    let trash_piece = trash
        .query_selector(&format!(".hidden[data-piece=\"{}\"]", data_piece))
        .expect("Not able to perform query selector");
    let trash_piece = trash_piece.unwrap_or_else(|| piece.clone());
    trash_piece.class_list_remove("hidden");

    if !(parent.id() == trash.id()) {
        trash.append_child(&trash_piece).unwrap();
    }

    piece.class_list_add("hidden");
    let data_square = piece.get_attribute("data-square").unwrap();
    trash_piece.class_list_remove(&format!("square-{}", data_square));
    trash_piece.class_list_add("deleted");

    trash_piece.remove_style("transform");
    trash_piece.remove_style("transition");
}

pub fn restore_piece(piece: &Element, client_position: (f64, f64)) {
    let mut parent = piece.parent_element().unwrap();
    let chess_board = piece.parent_element().unwrap().parent_element().unwrap();
    let board_piece = chess_board
        .query_selector(&format!(".hidden[data-piece=\"{}\"]", piece.id()))
        .expect("Not able to perform query selector");
    let board_piece = board_piece.unwrap_or_else(|| piece.clone());
    board_piece.class_list_remove("hidden");

    if !(parent.id() == "chessboard") {
        chess_board.append_child(&board_piece).unwrap();
        board_piece.class_list_remove("deleted");
        parent = board_piece.parent_element().unwrap();
    };
    piece.class_list_add("hidden");
    let translate_value = translate_value(client_position, &parent);
    board_piece.set_style("transform", &translate_value);
    board_piece.set_style("transition", "none");
}

fn is_piece_dark_variant(piece: &Element) -> bool {
    let data_piece = piece.get_attribute("data-piece").unwrap();
    let first_char = data_piece.chars().next().unwrap();

    match first_char {
        'l' => false,
        'd' => true,
        _ => unreachable!("Invalid piece data attribute"),
    }
}

fn transform_value(client_position: (f64, f64), elem: &Element) -> Transform {
    let bounding = elem.get_bounding_client_rect();
    let (x, y) = mouse_position_in_bounding(client_position, &bounding);
    let translate_x = x - 50.0;
    let translate_y = y - 50.0;
    Transform::new(translate_x, translate_y)
}

fn translate_value(client_position: (f64, f64), elem: &Element) -> String {
    let bounding = elem.get_bounding_client_rect();
    let (x, y) = mouse_position_in_bounding(client_position, &bounding);
    let translate_x = x - 50.0;
    let translate_y = y - 50.0;
    format!("translate({}%, {}%)", translate_x, translate_y)
}

pub fn mouse_in_bounding(client_position: (f64, f64), bounding: &DomRect) -> bool {
    let x = client_position.0;
    let y = client_position.1;

    let left = bounding.left();
    let right = bounding.right();
    let top = bounding.top();
    let bottom = bounding.bottom();

    x > left && x < right && y > top && y < bottom
}

pub fn mouse_position_in_bounding(client_position: (f64, f64), bounding: &DomRect) -> (f64, f64) {
    let max_x = bounding.width();
    let max_y = bounding.height();

    let x = match client_position.0 - bounding.left() {
        x if x < 0.0 => 0.0,
        x if x > max_x => max_x,
        x => x,
    };
    let y = match client_position.1 - bounding.top() {
        y if y < 0.0 => 0.0,
        y if y > max_y => max_y,
        y => y,
    };

    let x_percent = x * 800.0 / max_x;
    let y_percent = y * 800.0 / max_y;

    (x_percent, y_percent)
}
