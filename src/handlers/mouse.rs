use leptos::*;
use log;
use web_sys::{Element, MouseEvent, TouchEvent};

use crate::entities::position::Position;
use crate::utils::class_list::ClassListExt;
use crate::utils::elements;
use crate::utils::style::StyleExt;

// Only works in the browser

fn select_piece_square(piece: &Element) {
    let square = piece.get_attribute("data-square").unwrap();
    let selected_square = elements::generate::selected_square(&square);
    log::debug!("Selected square: {:?}", selected_square);
    elements::add_to_board(&selected_square);
    log::debug!("Selected square: {}", square);
}

pub fn mousemove(event: MouseEvent) {
    if let Some(piece) = elements::query_selector(".dragging") {
        let client_position = (event.client_x() as f64, event.client_y() as f64);
        elements::move_piece(&piece, client_position);
    }
}

pub fn touchmove(event: TouchEvent) {
    if let Some(piece) = elements::query_selector(".dragging") {
        let client_position = (10.1, 10.1);
        elements::move_piece(&piece, client_position);
    }
}

pub fn mousedown(event: MouseEvent) {
    // Add dragging class to the piece
    let piece = elements::event_target_elem(&event);
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

pub fn mouseup(event: MouseEvent) {
    // let valid_move = true;

    // Remove dragging class from the piece
    if let Some(piece) = elements::query_selector(".dragging") {
        piece.remove_style("transform");
        piece.remove_style("transition");

        let chess_board = piece.parent_element().unwrap();
        let bounding = chess_board.get_bounding_client_rect();
        let client_position = (event.client_x() as f64, event.client_y() as f64);
        let (x, y) = elements::mouse_position_in_bounding(client_position, &bounding);
        let is_white_view = !chess_board.class_list_include("flipped");
        let position = Position::from_ui_position(x, y, is_white_view);

        let old_square = piece.get_attribute("data-square").unwrap();
        piece
            .set_attribute("data-square", &position.to_string())
            .unwrap();
        piece.class_list_remove(&format!("square-{}", old_square));
        piece.class_list_add(&format!("square-{}", position.to_string()));

        // Remove the hovered class from the square
        if let Some(square) = elements::query_selector(".dragging-over") {
            square.class_list_remove("dragging-over");
        }

        piece.class_list_remove("dragging");
    }
}

pub fn mouseover(_event: MouseEvent) {}
