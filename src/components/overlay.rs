use std::time::Duration;

use leptos::prelude::*;
use leptos_router::hooks::{use_location, use_navigate};

use crate::{
    components::{
        check_mate::CheckMate,
        forms::{room_path, Form, Forms},
        game_banner::GameBanner,
        menu::Menu,
        notifications::Notifications,
        people::PeopleDropdown,
        promotion::PromotionPicker,
    },
    entities::{
        chess_board::signals::{ChessBoardSignals, Connection},
        protocol::{ClientMsg, Side, MAIN_ROOM},
    },
    utils::{browser::now_ms, get_user_payload, WindowExt},
};

/// The room of a page: `/g/<room>`, or the main board at `/`
pub fn room_from_path(path: &str) -> String {
    path.strip_prefix("/g/")
        .map(|room| room.trim_end_matches('/'))
        .filter(|room| !room.is_empty())
        .unwrap_or(MAIN_ROOM)
        .to_string()
}

/// Everything layered over the board: menus, dialogs, banners, and the
/// effects that keep the page in step with the URL and the socket
#[component]
pub fn Overlay(
    chess_board_signals: ChessBoardSignals,
    show_form: RwSignal<Form>,
    menu_open: RwSignal<bool>,
    people_open: RwSignal<bool>,
) -> impl IntoView {
    let location = use_location();
    let navigate = use_navigate();

    Effect::new(move |_| {
        if let Some(_) = get_user_payload().map(|payload| payload.name) {
            chess_board_signals.start_websocket();
        } else {
            let window = web_sys::window().unwrap();
            window.set_timeout_callback(move || show_form.set(Form::Username), 0);
        };
    });

    Effect::new(move |_| {
        let room = room_from_path(&location.pathname.get());
        chess_board_signals.room_target().set(room);
    });

    // joins the page's room once the socket is open, and again whenever
    // either changes
    Effect::new(move |_| {
        let room = chess_board_signals.room_target().get();
        if chess_board_signals.connection().get() == Connection::Open && !room.is_empty() {
            chess_board_signals.send(&ClientMsg::Join {
                room,
                fen: None,
                trash: None,
            });
        }
    });

    // the server refused the session cookie
    Effect::new(move |_| {
        if chess_board_signals.needs_username().get() {
            chess_board_signals.needs_username().set(false);
            show_form.set(Form::Username);
        }
    });

    let navigate_to_game = navigate.clone();
    Effect::new(move |_| {
        if let Some(room) = chess_board_signals.created_game().get() {
            chess_board_signals.created_game().set(None);
            navigate_to_game(&room_path(&room), Default::default());
            show_form.set(Form::Invite);
        }
    });

    Effect::new(move |_| {
        if let Some(room) = chess_board_signals.redirect().get() {
            chess_board_signals.redirect().set(None);
            navigate(&room_path(&room), Default::default());
        }
    });

    // a player sees the board from their side, and white's again once they
    // leave the game
    Effect::new(move |previous: Option<Option<Side>>| {
        let seat = chess_board_signals.my_seat();
        if seat != previous.flatten() {
            chess_board_signals
                .chess_board()
                .update(|board| board.is_white_view = seat != Some(Side::Black));
        }
        seat
    });

    // ticks while a clock runs, for the countdown
    let ticker = StoredValue::new(None::<IntervalHandle>);
    Effect::new(move |_| {
        let running = chess_board_signals.game().with(|game| {
            game.as_ref()
                .and_then(|game| game.clock)
                .is_some_and(|clock| clock.running.is_some())
        });

        match (running, ticker.get_value()) {
            (true, None) => {
                chess_board_signals.clock_now().set(now_ms());
                let tick = move || chess_board_signals.clock_now().set(now_ms());
                ticker.set_value(set_interval_with_handle(tick, Duration::from_millis(100)).ok());
            }
            (false, Some(handle)) => {
                handle.clear();
                ticker.set_value(None);
            }
            _ => {}
        }
    });

    let any_dropdown = move || menu_open.get() || people_open.get();
    let close_dropdowns = move |_| {
        menu_open.set(false);
        people_open.set(false);
    };

    view! {
        <>
            <Notifications chess_board_signals=chess_board_signals />
            <Show when=any_dropdown>
                <div class="dropdown-backdrop" on:click=close_dropdowns></div>
            </Show>
            <Menu show_form=show_form chess_board_signals=chess_board_signals open=menu_open />
            <PeopleDropdown chess_board_signals=chess_board_signals open=people_open show_form=show_form />
            <GameBanner chess_board_signals=chess_board_signals show_form=show_form />
            <Forms show_form=show_form chess_board_signals=chess_board_signals />
            <PromotionPicker chess_board_signals=chess_board_signals />
            <CheckMate chess_board_signals=chess_board_signals show_form=show_form />
        </>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_from_path() {
        assert_eq!(room_from_path("/"), MAIN_ROOM);
        assert_eq!(room_from_path("/g/abc123"), "abc123");
        assert_eq!(room_from_path("/g/abc123/"), "abc123");
        assert_eq!(room_from_path("/g/"), MAIN_ROOM);
    }
}
