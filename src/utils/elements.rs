use web_sys::{Document, DomRect, Element, Node};

use super::class_list::ClassListExt;

use super::{js_cast, style::StyleExt};

pub fn document() -> Document {
    web_sys::window().unwrap().document().unwrap()
}

pub fn query_selector(class: &str) -> Option<Element> {
    document().query_selector(class).unwrap()
}

pub fn move_piece(piece: &Element, client_position: (f64, f64)) {
    let parent = piece.parent_element().unwrap();
    let data_square = piece.get_attribute("data-square").unwrap();
    let data_deleted = piece
        .get_attribute("data-deleted")
        .unwrap()
        .parse::<bool>()
        .unwrap();

    let dark_trash = query_selector("[data-trash=\"dark\"]").unwrap();
    let light_trash = query_selector("[data-trash=\"light\"]").unwrap();
    let chess_board = query_selector("#chessboard").unwrap();
    dark_trash.class_list_add("dragging-over");
    light_trash.class_list_add("dragging-over");
    chess_board.class_list_add("dragging-over");
    let in_dark_trash = mouse_in_bounding(client_position, &dark_trash.get_bounding_client_rect());
    let in_light_trash =
        mouse_in_bounding(client_position, &light_trash.get_bounding_client_rect());

    if data_square == "deleted" {
        let restored_piece = if let Some(piece) = find_restored_piece_clone(piece) {
            piece
        } else {
            let piece = create_restored_piece_clone(piece).unwrap();
            chess_board.append_child(&piece).unwrap();
            piece
        };

        if in_dark_trash || in_light_trash {
            restored_piece.class_list_add("hidden");
            piece.class_list_remove("hidden");
            if !data_deleted {
                piece.set_attribute("data-deleted", "true").unwrap();
            }
        } else {
            restored_piece.class_list_remove("hidden");
            piece.class_list_add("hidden");
            if data_deleted {
                piece.set_attribute("data-deleted", "false").unwrap();
            }

            restored_piece.set_style("transition", "none");
            restored_piece.set_style("transform", &translate_value(client_position, &chess_board));
        }
    } else {
        let deleted_piece = if let Some(piece) = find_deleted_piece_clone(piece) {
            piece
        } else {
            let piece = create_deleted_piece_clone(piece).unwrap();
            if is_piece_dark_variant(&piece) {
                dark_trash.append_child(&piece).unwrap();
            } else {
                light_trash.append_child(&piece).unwrap();
            }
            piece
        };

        if in_dark_trash || in_light_trash {
            piece.class_list_add("hidden");
            deleted_piece.class_list_remove("hidden");
            if !data_deleted {
                piece.set_attribute("data-deleted", "true").unwrap();
            }
        } else {
            piece.class_list_remove("hidden");
            deleted_piece.class_list_add("hidden");
            if data_deleted {
                piece.set_attribute("data-deleted", "false").unwrap();
            }

            piece.set_style("transition", "none");
            piece.set_style("transform", &translate_value(client_position, &parent));
        }
    }
}

pub fn find_restored_piece_clone(piece: &Element) -> Option<Element> {
    let data_square = piece.get_attribute("data-square").unwrap();
    if data_square != "deleted" {
        return None;
    }

    let data_key = piece.get_attribute("data-key").unwrap();
    query_selector(&format!(".restored[data-key=\"{}\"]", data_key))
}

pub fn create_restored_piece_clone(piece: &Element) -> Option<Element> {
    if let Some(piece) = js_cast::<Element, Node>(piece.clone_node().unwrap()) {
        piece.set_attribute("data-deleted", "false").unwrap();
        piece.class_list_remove("deleted");
        piece.class_list_add("restored");
        piece.class_list_remove("dragging");

        return Some(piece);
    }

    None
}

pub fn delete_restored_piece_clone(piece: &Element) {
    if let Some(restored_piece) = find_restored_piece_clone(piece) {
        restored_piece.remove();
    };
}

pub fn find_deleted_piece_clone(piece: &Element) -> Option<Element> {
    let data_square = piece.get_attribute("data-square").unwrap();
    if data_square == "deleted" {
        return None;
    }

    let data_key = piece.get_attribute("data-key").unwrap();
    query_selector(&format!(".deleted[data-key=\"{}\"]", data_key))
}

pub fn create_deleted_piece_clone(piece: &Element) -> Option<Element> {
    if let Some(piece) = js_cast::<Element, Node>(piece.clone_node().unwrap()) {
        let data_square = piece.get_attribute("data-square").unwrap();
        piece.set_attribute("data-deleted", "true").unwrap();
        piece.class_list_add("deleted");
        piece.class_list_remove("dragging");
        piece.class_list_remove(&format!("square-{}", data_square));
        piece.remove_style("transform");
        piece.remove_style("transition");

        return Some(piece);
    };
    None
}

pub fn delete_deleted_piece_clone(piece: &Element) {
    if let Some(deleted_piece) = find_deleted_piece_clone(piece) {
        deleted_piece.remove();
    };
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
