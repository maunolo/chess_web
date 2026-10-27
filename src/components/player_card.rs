use leptos::prelude::*;

use crate::{
    components::trash::{Trash, TrashType},
    entities::{chess_board::signals::ChessBoardSignals, protocol::Side},
    utils::my_id,
};

/// `m:ss`, `h:mm:ss` from an hour on, and tenths under ten seconds
pub fn format_clock(ms: i64) -> String {
    let ms = ms.max(0);
    let secs = ms / 1000;
    let (hours, minutes, seconds) = (secs / 3600, secs / 60 % 60, secs % 60);

    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else if ms < 10_000 {
        format!("{minutes}:{seconds:02}.{}", ms % 1000 / 100)
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

/// A player's name, captured pieces and clock, above or below the board
#[component]
pub fn PlayerCard(chess_board_signals: ChessBoardSignals, side: Side) -> impl IntoView {
    let side_name = match side {
        Side::White => "White",
        Side::Black => "Black",
    };
    // a card sits on its own side of the board, so white's goes at the
    // bottom of a white view and at the top of a flipped one
    let order = move || {
        let white_view = chess_board_signals.chess_board().with(|board| board.white_view());
        let at_bottom = (side == Side::White) == white_view;
        if at_bottom { "order: 2" } else { "order: 0" }
    };

    let player_id = move || {
        chess_board_signals
            .game()
            .with(|game| game.as_ref().map(|game| game.player(side).map(str::to_string)))
    };
    let name = move || match player_id() {
        None => side_name.to_string(),
        Some(None) => "Waiting…".to_string(),
        Some(Some(id)) => chess_board_signals
            .room_status()
            .with(|status| status.as_ref().and_then(|status| status.get_user(&id)))
            .map(|user| user.with(|user| user.username()))
            .unwrap_or("Player".to_string()),
    };
    let initial = move || match player_id() {
        Some(None) => "?".to_string(),
        _ => name()
            .chars()
            .next()
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_default(),
    };
    let is_me = move || player_id().flatten() == my_id();

    let game_over = move || {
        chess_board_signals
            .game()
            .with(|game| game.as_ref().is_some_and(|game| game.result.is_some()))
    };
    let to_move = move || {
        let turn = chess_board_signals.chess_board().with(|board| board.turn) == side;
        let shows_turns = chess_board_signals.is_game() || chess_board_signals.validation();
        turn && shows_turns && !game_over()
    };
    let card_class = move || {
        if to_move() {
            "player-card player-card--to-move"
        } else {
            "player-card"
        }
    };
    let card_label = move || match (to_move(), is_me()) {
        (true, true) => format!("{side_name}, your move"),
        (true, false) => format!("{side_name}, to move"),
        (false, _) => side_name.to_string(),
    };

    // counts down locally from the last time the server sent the clock
    let clock_ms = move || {
        let clock = chess_board_signals
            .game()
            .with(|game| game.as_ref().and_then(|game| game.clock))?;
        let mut ms = clock.remaining_ms(side);
        if clock.running == Some(side) {
            let elapsed =
                chess_board_signals.clock_now().get() - chess_board_signals.clock_synced_at().get();
            ms -= elapsed.max(0.0) as i64;
        }
        Some((ms.max(0), clock.running == Some(side)))
    };
    let clock_class = move || match clock_ms() {
        Some((ms, true)) if ms < 10_000 => "clock clock--low",
        Some((_, true)) => "clock clock--running",
        _ => "clock",
    };

    // white captures the dark pieces, so they show on white's card
    let trash = match side {
        Side::White => TrashType::Dark,
        Side::Black => TrashType::Light,
    };
    let deleted_stones = move || {
        chess_board_signals
            .stones_signals()
            .get()
            .deleted_stones()
            .clone()
    };

    view! {
        <div class=card_class style=order aria-label=card_label>
            <div aria-hidden="true" class="avatar">{initial}</div>
            <div class="flex-grow min-w-0 flex flex-col gap-0.5">
                <span class="player-name">{name}</span>
                <div class="player-status">
                    <Trash chess_board_signals=chess_board_signals id=trash trash=deleted_stones/>
                </div>
            </div>
            {move || {
                clock_ms()
                    .map(|(ms, _)| view! { <div class=clock_class>{format_clock(ms)}</div> })
            }}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_clock() {
        assert_eq!(format_clock(300_000), "5:00");
        assert_eq!(format_clock(65_900), "1:05");
        assert_eq!(format_clock(9_450), "0:09.4");
        assert_eq!(format_clock(3_725_000), "1:02:05");
        assert_eq!(format_clock(-5), "0:00.0");
    }
}
