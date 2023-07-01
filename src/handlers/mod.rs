use leptos::{RwSignal, SignalGet};

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

// Only works in the browser

// Possible events:
// fn select_piece_square(piece: &Element) {
//     let square = piece.get_attribute("data-square").unwrap();
//     let selected_square = elements::generate::selected_square(&square);
//     log::debug!("Selected square: {:?}", selected_square);
//     elements::add_to_board(&selected_square);
//     log::debug!("Selected square: {}", square);
// }

pub fn interaction_move<E>(event: E)
where
    E: EventPositionExt,
{
    if let Some(piece) = elements::query_selector(".dragging") {
        let position = event.position();
        let client_position = (position.0 as f64, position.1 as f64);
        elements::move_piece(&piece, client_position);
    }
}

pub fn interaction_start<E>(event: E)
where
    E: EventPositionExt + EventTargetExt,
{
    let Some(piece) = event.target_element() else {
        log::debug!("No piece found");
        return;
    };
    // select_piece_square(&piece);
    piece.class_list_add("dragging");

    // Add dragging-over class to the board
    if let Some(square) = piece.parent_element() {
        square.class_list_add("dragging-over");
    }

    let position = event.position();

    // Move the piece to client cursor position
    let client_position = (position.0 as f64, position.1 as f64);
    elements::move_piece(&piece, client_position);
}

pub fn interaction_end<E>(event: E) -> Result<(String, (String, String))>
where
    E: EventPositionExt,
{
    if let Some(piece) = elements::query_selector(".dragging") {
        piece.remove_style("transform");
        piece.remove_style("transition");

        let old_square = piece.get_attribute("data-square").unwrap();
        let old_pos = old_square.clone();
        let new_pos: String;

        if !piece.class_list_include("deleted") {
            let chess_board = piece.parent_element().unwrap();
            let bounding = chess_board.get_bounding_client_rect();
            let position = event.position();

            let client_position = (position.0 as f64, position.1 as f64);
            let (x, y) = mouse_position_in_bounding(client_position, &bounding);
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

pub fn interaction_end_with_websocket<E>(websocket: RwSignal<Option<web_sys::WebSocket>>, event: E)
where
    E: EventPositionExt,
{
    if let Ok((piece_data, (old_pos, new_pos))) = interaction_end(event) {
        if let Some(socket) = websocket.get().as_ref() {
            let msg = format!("/move {} {} {}", piece_data, old_pos, new_pos);

            match socket.send_with_str(&msg) {
                Ok(_) => log::debug!("message successfully sent: {:?}", msg),
                Err(err) => log::debug!("error sending message: {:?}", err),
            }
        }
    };
}
