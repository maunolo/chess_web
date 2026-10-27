use leptos::prelude::*;

use crate::{
    entities::chess_board::signals::ChessBoardSignals,
    i18n::{self, Key},
};

/// `(number, white's move, black's move)` rows
fn pairs(moves: &[String]) -> Vec<(usize, String, Option<String>)> {
    moves
        .chunks(2)
        .enumerate()
        .map(|(i, pair)| (i + 1, pair[0].clone(), pair.get(1).cloned()))
        .collect()
}

/// The last few moves on one line, for the phone
#[component]
pub fn MoveStrip(chess_board_signals: ChessBoardSignals) -> impl IntoView {
    let rows = move || {
        let moves = chess_board_signals.moves().get();
        let count = moves.len();
        let mut rows = pairs(&moves);
        let shown = rows.len().min(5);
        rows.drain(..rows.len() - shown);
        rows.into_iter()
            .map(|(no, white, black)| {
                let white_last = black.is_none() && no * 2 - 1 == count;
                let black_last = black.is_some() && no * 2 == count;
                (no, white, black, white_last, black_last)
            })
            .collect::<Vec<_>>()
    };

    view! {
        <div class="move-strip lg:hidden">
            // the highlight is part of the key, so a row re-renders when it stops being last
            <For
                each=rows
                key=|row| row.clone()
                children=|(no, white, black, white_last, black_last)| {
                    let white_class = if white_last { "move--last" } else { "" };
                    let black_class = if black_last { "move--last" } else { "" };
                    view! {
                        <span class="move-no">{format!("{no}.")}</span>
                        <span class=white_class>{white}</span>
                        {black.map(|black| view! { <span class=black_class>{black}</span> })}
                    }
                }
            />
        </div>
    }
}

/// Every move so far, three columns, for the desktop panel
#[component]
pub fn MoveTable(chess_board_signals: ChessBoardSignals) -> impl IntoView {
    let lang = i18n::current();
    let rows = move || {
        let moves = chess_board_signals.moves().get();
        let count = moves.len();
        pairs(&moves)
            .into_iter()
            .map(|(no, white, black)| {
                let white_last = no * 2 - 1 == count;
                let black_last = no * 2 == count;
                (no, white, black, white_last, black_last)
            })
            .collect::<Vec<_>>()
    };
    let empty = move || chess_board_signals.moves().with(|moves| moves.is_empty());

    view! {
        <div class="card p-4 flex flex-col gap-2.5">
            <span class="eyebrow">{lang.t(Key::Moves)}</span>
            <Show when=empty>
                <span class="text-sm text-ink-300">{lang.t(Key::NoMovesYet)}</span>
            </Show>
            <div class="move-table">
                <For
                    each=rows
                    key=|row| row.clone()
                    children=|(no, white, black, white_last, black_last)| {
                        let white_class = if white_last { "move--last" } else { "" };
                        let black_class = if black_last { "move--last" } else { "" };
                        view! {
                            <span class="move-no">{format!("{no}.")}</span>
                            <span class=white_class>{white}</span>
                            <span class=black_class>{black.unwrap_or_default()}</span>
                        }
                    }
                />
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pairs() {
        let moves: Vec<String> = ["e4", "e5", "Nf3"].map(String::from).to_vec();
        assert_eq!(
            pairs(&moves),
            vec![
                (1, "e4".to_string(), Some("e5".to_string())),
                (2, "Nf3".to_string(), None)
            ]
        );
        assert!(pairs(&[]).is_empty());
    }
}
