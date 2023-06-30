use web_sys::{MouseEvent, TouchEvent};

use crate::entities::position::Position;
use crate::utils::class_list::ClassListExt;
use crate::utils::elements;
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

// Only works in the browser

// Possible events:
// fn select_piece_square(piece: &Element) {
//     let square = piece.get_attribute("data-square").unwrap();
//     let selected_square = elements::generate::selected_square(&square);
//     log::debug!("Selected square: {:?}", selected_square);
//     elements::add_to_board(&selected_square);
//     log::debug!("Selected square: {}", square);
// }

pub fn mousemove(event: MouseEvent) {
    if let Some(piece) = elements::query_selector(".dragging") {
        let client_position = (event.client_x() as f64, event.client_y() as f64);
        elements::move_piece(&piece, client_position);
    }
}

pub fn touchmove(event: TouchEvent) {
    if let Some(piece) = elements::query_selector(".dragging") {
        let touches = event.touches();
        if let Some(touch) = touches.get(0) {
            let client_position = (touch.client_x() as f64, touch.client_y() as f64);
            elements::move_piece(&piece, client_position);
        }
    }
}

pub fn mousedown(event: MouseEvent) {
    // Add dragging class to the piece
    let Some(piece) = elements::event_target_elem(&event) else {
        log::debug!("No piece found");
        return;
    };
    // select_piece_square(&piece);
    piece.class_list_add("dragging");

    // Add dragging-over class to the board
    if let Some(square) = piece.parent_element() {
        square.class_list_add("dragging-over");
    }

    // Move the piece to client cursor position
    let client_position = (event.client_x() as f64, event.client_y() as f64);
    elements::move_piece(&piece, client_position);
}

pub fn touchstart(event: TouchEvent) {
    let Some(piece) = elements::event_target_elem(&event) else {
        log::debug!("No piece found");
        return;
    };
    // select_piece_square(&piece);
    piece.class_list_add("dragging");

    // Add dragging-over class to the board
    if let Some(square) = piece.parent_element() {
        square.class_list_add("dragging-over");
    }

    let touches = event.touches();
    if let Some(touch) = touches.get(0) {
        // Move the piece to client cursor position
        let client_position = (touch.client_x() as f64, touch.client_y() as f64);
        elements::move_piece(&piece, client_position);
    }
}

pub fn mouseup(event: MouseEvent) -> Result<(String, (String, String))> {
    // let valid_move = true;

    // Remove dragging class from the piece
    if let Some(piece) = elements::query_selector(".dragging") {
        piece.remove_style("transform");
        piece.remove_style("transition");

        let old_square = piece.get_attribute("data-square").unwrap();
        let old_pos = old_square.clone();
        let new_pos: String;

        if !piece.class_list_include("deleted") {
            let chess_board = piece.parent_element().unwrap();
            let bounding = chess_board.get_bounding_client_rect();
            let client_position = (event.client_x() as f64, event.client_y() as f64);
            let (x, y) = elements::mouse_position_in_bounding(client_position, &bounding);
            let is_white_view = !chess_board.class_list_include("flipped");
            let position = Position::from_ui_position(x, y, is_white_view);

            if position.to_string() != old_square {
                if let Some(old_piece) =
                    elements::query_selector(&format!("[data-square=\"{}\"]", position.to_string()))
                {
                    elements::soft_delete_piece(&old_piece);
                    old_piece.set_attribute("data-square", "deleted").unwrap();
                }
                piece
                    .set_attribute("data-square", &position.to_string())
                    .unwrap();
                piece.class_list_remove(&format!("square-{}", old_square));
            }
            piece.class_list_add(&format!("square-{}", position.to_string()));
            new_pos = position.to_string();
        } else {
            piece.set_attribute("data-square", "deleted").unwrap();
            new_pos = "deleted".to_string();
        }

        // Remove the hovered class from the square
        if let Some(square) = elements::query_selector(".dragging-over") {
            square.class_list_remove("dragging-over");
        }

        piece.class_list_remove("dragging");
        Ok((
            piece.get_attribute("data-piece").unwrap(),
            (old_pos, new_pos),
        ))
    } else {
        Err(EventError::new("No piece being dragged found"))
    }
}

pub fn touchend(event: TouchEvent) -> Result<(String, (String, String))> {
    if let Some(piece) = elements::query_selector(".dragging") {
        piece.remove_style("transform");
        piece.remove_style("transition");

        let old_square = piece.get_attribute("data-square").unwrap();
        let old_pos = old_square.clone();
        let new_pos: String;

        if !piece.class_list_include("deleted") {
            let chess_board = piece.parent_element().unwrap();
            let bounding = chess_board.get_bounding_client_rect();
            let touches = event.changed_touches();
            if let Some(touch) = touches.get(touches.length() - 1) {
                let client_position = (touch.client_x() as f64, touch.client_y() as f64);
                let (x, y) = elements::mouse_position_in_bounding(client_position, &bounding);
                let is_white_view = !chess_board.class_list_include("flipped");
                let position = Position::from_ui_position(x, y, is_white_view);

                if position.to_string() != old_square {
                    if let Some(old_piece) = elements::query_selector(&format!(
                        "[data-square=\"{}\"]",
                        position.to_string()
                    )) {
                        elements::soft_delete_piece(&old_piece);
                        old_piece.set_attribute("data-square", "deleted").unwrap();
                    }
                    piece
                        .set_attribute("data-square", &position.to_string())
                        .unwrap();
                    piece.class_list_remove(&format!("square-{}", old_square));
                }
                piece.class_list_add(&format!("square-{}", position.to_string()));
                new_pos = position.to_string();
            } else {
                return Err(EventError::new("No touch found"));
            }
        } else {
            piece.set_attribute("data-square", "deleted").unwrap();
            new_pos = "deleted".to_string();
        }

        // Remove the hovered class from the square
        if let Some(square) = elements::query_selector(".dragging-over") {
            square.class_list_remove("dragging-over");
        }

        piece.class_list_remove("dragging");
        Ok((
            piece.get_attribute("data-piece").unwrap(),
            (old_pos, new_pos),
        ))
    } else {
        Err(EventError::new("No piece being dragged found"))
    }
}
