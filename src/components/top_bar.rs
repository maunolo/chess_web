use leptos::prelude::*;

use crate::{
    components::{forms::Form, icons::Icon, people::PeopleButton},
    entities::{chess_board::signals::ChessBoardSignals, protocol::TimeControl},
};

fn time_control_category(time_control: &TimeControl) -> &'static str {
    match time_control.initial_secs {
        secs if secs < 180 => "Bullet",
        secs if secs < 600 => "Blitz",
        secs if secs < 1800 => "Rapid",
        _ => "Classical",
    }
}

/// What the page is showing: a title and a line under it
pub fn room_title(chess_board_signals: ChessBoardSignals) -> (String, String) {
    let room = chess_board_signals
        .room_status()
        .with(|status| status.as_ref().map(|status| status.name()))
        .unwrap_or("Chess".to_string());
    let game = chess_board_signals.game().get();

    match game {
        Some(game) => {
            let title = match game.time_control {
                Some(time_control) => format!(
                    "{} · {}",
                    time_control.label(),
                    time_control_category(&time_control)
                ),
                None => "Casual game".to_string(),
            };
            (title, format!("Game {room}"))
        }
        None => {
            let mut title = room;
            if let Some(first) = title.get_mut(..1) {
                first.make_ascii_uppercase();
            }
            (title, "Free board".to_string())
        }
    }
}

#[component]
pub fn TopBar(
    chess_board_signals: ChessBoardSignals,
    show_form: RwSignal<Form>,
    menu_open: RwSignal<bool>,
    people_open: RwSignal<bool>,
) -> impl IntoView {
    let title = move || room_title(chess_board_signals).0;
    let subtitle = move || room_title(chess_board_signals).1;
    let menu_btn_class = move || {
        if menu_open.get() {
            "icon-btn menu-btn menu-btn--is-active"
        } else {
            "icon-btn menu-btn"
        }
    };
    let toggle_menu = move |_| {
        people_open.set(false);
        menu_open.update(|open| *open = !*open);
    };

    view! {
        // above the dropdown backdrop, so its buttons switch between dropdowns
        <header class="relative z-40 h-14 flex-shrink-0 flex items-center justify-between px-2 lg:px-5 lg:border-b lg:border-ink-800">
            <div class="flex items-center gap-2 lg:hidden">
                <button class=menu_btn_class on:click=toggle_menu aria-label="Menu">
                    <span class="line"></span>
                </button>
            </div>
            <div class="hidden lg:flex items-center gap-2.5">
                <span aria-hidden="true" class="w-7 h-7 rounded-lg bg-accent"></span>
                <span class="text-base font-bold">Chess</span>
            </div>

            <div class="flex flex-col items-center min-w-0 lg:hidden">
                <span class="text-[15px] font-semibold truncate max-w-[12rem]">{title}</span>
                <span class="text-xs text-ink-300 truncate max-w-[12rem]">{subtitle}</span>
            </div>

            <div class="flex items-center gap-1 lg:gap-2">
                <button class="btn btn-outline hidden lg:inline-flex h-10" on:click=move |_| show_form.set(Form::NewGame)>
                    "New game"
                </button>
                <button class="btn btn-outline hidden lg:inline-flex h-10" on:click=move |_| show_form.set(Form::Join)>
                    "Join room"
                </button>
                <button class="btn btn-primary hidden lg:inline-flex h-10" on:click=move |_| show_form.set(Form::Invite)>
                    "Invite"
                </button>
                <Show when=move || chess_board_signals.is_game()>
                    <button class="icon-btn lg:hidden" aria-label="Invite" on:click=move |_| show_form.set(Form::Invite)>
                        {Icon::Invite.view()}
                    </button>
                </Show>
                <PeopleButton chess_board_signals=chess_board_signals open=people_open menu_open=menu_open/>
            </div>
        </header>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_control_category() {
        let category = |initial_secs| {
            time_control_category(&TimeControl {
                initial_secs,
                increment_secs: 0,
            })
        };
        assert_eq!(category(60), "Bullet");
        assert_eq!(category(180), "Blitz");
        assert_eq!(category(600), "Rapid");
        assert_eq!(category(1800), "Classical");
    }
}
