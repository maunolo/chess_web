use leptos::prelude::*;

use crate::entities::{chess_board::signals::ChessBoardSignals, protocol::Side};

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

/// A player's name and clock, shown on the board's edge during a game
#[component]
pub fn PlayerChip(chess_board_signals: ChessBoardSignals, side: Side) -> impl IntoView {
    let name = move || {
        let player = chess_board_signals
            .game()
            .with(|game| game.as_ref().map(|game| game.player(side).map(str::to_string)))?;

        Some(match player {
            None => "Waiting…".to_string(),
            Some(id) => chess_board_signals
                .room_status()
                .with(|status| status.as_ref().and_then(|status| status.get_user(&id)))
                .map(|user| user.with(|user| user.username()))
                .unwrap_or("Player".to_string()),
        })
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
        Some(ms.max(0))
    };
    let to_move = move || {
        chess_board_signals
            .game()
            .with(|game| game.as_ref().is_some_and(|game| game.result.is_none()))
            && chess_board_signals.chess_board().with(|board| board.turn) == side
    };

    move || {
        name().map(|name| {
            view! {
                <div class=move || {
                    if to_move() { "player-chip player-chip--to-move" } else { "player-chip" }
                }>
                    <span class="max-w-24 truncate">{name}</span>
                    {move || {
                        clock_ms()
                            .map(|ms| {
                                let class = if ms < 10_000 { "clock clock--low" } else { "clock" };
                                view! { <span class=class>{format_clock(ms)}</span> }
                            })
                    }}
                </div>
            }
        })
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
