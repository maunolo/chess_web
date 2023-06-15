use wasm_bindgen::JsCast;
use web_sys::{Document, DomRect, Element, Event};

use super::class_list::ClassListExt;
use super::style::StyleExt;

pub mod generate {
    use crate::utils::element_pool::ElementPoolExt;
    use web_sys::Element;

    pub fn selected_square(square: &str) -> Element {
        let elem = Element::find_or_create();
        elem.set_attribute("data-square", square).unwrap();
        elem.set_attribute("class", &format!("selected square-{}", square))
            .unwrap();
        elem
    }
}

pub fn document() -> Document {
    web_sys::window().unwrap().document().unwrap()
}

pub fn event_target_elem(event: &Event) -> Element {
    let target = event.target().unwrap();
    JsCast::dyn_ref::<Element>(&target).unwrap().to_owned()
}

pub fn query_selector(class: &str) -> Option<Element> {
    document().query_selector(class).unwrap()
}

pub fn add_to_board(elem: &Element) {
    let already_in_board = elem
        .parent_element()
        .and_then(|e| Some(e.class_list_include("chessboard")));
    match already_in_board {
        Some(true) => {}
        Some(false) | None => {
            let chess_board = query_selector(".chessboard").unwrap();
            chess_board.append_child(elem).unwrap();
        }
    }
}

pub fn move_piece(piece: &Element, client_position: (f64, f64)) {
    let mut parent = piece.parent_element().unwrap();
    let dark_trash = query_selector("[data-trash=\"dark\"]").unwrap();
    let light_trash = query_selector("[data-trash=\"light\"]").unwrap();
    let in_dark_trash = mouse_in_bounding(client_position, &dark_trash.get_bounding_client_rect());
    let in_light_trash =
        mouse_in_bounding(client_position, &light_trash.get_bounding_client_rect());

    if in_dark_trash || in_light_trash {
        soft_delete_piece(piece);
    } else {
        log::debug!("parent ID: {}", parent.id());
        if !(parent.id() == "chessboard") {
            restore_piece(piece);
            parent = piece.parent_element().unwrap();
        };
        let translate_value = translate_value(client_position, &parent);
        piece.set_style("transform", &translate_value);
        piece.set_style("transition", "none");
    }
}

pub fn soft_delete_piece(piece: &Element) {
    let parent = piece.parent_element().unwrap();
    let trash = if is_piece_dark_variant(&piece) {
        query_selector("[data-trash=\"dark\"]").unwrap()
    } else {
        query_selector("[data-trash=\"light\"]").unwrap()
    };

    if !(parent.id() == trash.id()) {
        trash.append_child(piece).unwrap();
    }

    let data_square = piece.get_attribute("data-square").unwrap();
    piece.class_list_remove(&format!("square-{}", data_square));
    piece.class_list_add("deleted");

    piece.remove_style("transform");
    piece.remove_style("transition");
}

pub fn restore_piece(piece: &Element) {
    let chess_board = piece.parent_element().unwrap().parent_element().unwrap();
    chess_board.append_child(piece).unwrap();
    piece.class_list_remove("deleted");
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
