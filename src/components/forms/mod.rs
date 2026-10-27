pub mod invite;
pub mod join;
pub mod new_game;
pub mod options;
pub mod username;

use crate::entities::chess_board::signals::ChessBoardSignals;
use crate::entities::protocol::{ClientMsg, SeatChoice, TimeControl, MAIN_ROOM};
use crate::utils::{sanitize::sanitize_room_name, WindowExt};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use web_sys::SubmitEvent;

use self::{invite::Invite, join::Join, new_game::NewGame, options::Options, username::Username};

const BACKDROP_CLASS: &str =
    "z-40 flex absolute w-full h-full justify-center items-center bg-neutral-900/30";

#[derive(Clone)]
#[allow(dead_code)]
pub enum Form {
    None,
    Join,
    Username,
    Options,
    NewGame,
    Invite,
}

/// The page of a room: the main board at `/`, the others at `/g/<room>`
pub fn room_path(room: &str) -> String {
    if room == MAIN_ROOM {
        "/".to_string()
    } else {
        format!("/g/{room}")
    }
}

fn set_username(username: &str, chess_board_signals: ChessBoardSignals) {
    let window = web_sys::window().unwrap();
    window.post_user_name(username, chess_board_signals);
}

fn form_data(e: &SubmitEvent) -> Option<web_sys::FormData> {
    let form = crate::utils::js_cast::<web_sys::HtmlFormElement, _>(e.target()?)?;
    web_sys::FormData::new_with_form(&form).ok()
}

/// "300+2" from the new game form
fn parse_time_control(value: &str) -> Option<TimeControl> {
    let (initial, increment) = value.split_once('+')?;
    Some(TimeControl {
        initial_secs: initial.parse().ok()?,
        increment_secs: increment.parse().ok()?,
    })
}

#[component]
pub fn Forms(chess_board_signals: ChessBoardSignals, show_form: RwSignal<Form>) -> impl IntoView {
    let navigate = use_navigate();

    let join_submit = move |e: SubmitEvent| {
        e.prevent_default();
        let Some(data) = form_data(&e) else {
            return;
        };
        let input = data.get("room").as_string().unwrap_or_default();
        // `<room> [<fen>[|<trash>]]`
        let (room, position) = input
            .trim()
            .split_once(' ')
            .unwrap_or((input.trim(), ""));
        let Some(room) = sanitize_room_name(room) else {
            return;
        };

        // a position only applies to a room that doesn't exist yet, so it's
        // sent before the page joins the room
        let position = position.trim();
        if !position.is_empty() {
            let (fen, trash) = match position.split_once('|') {
                Some((fen, trash)) => (fen, Some(trash.to_string())),
                None => (position, None),
            };
            chess_board_signals.send(&ClientMsg::Join {
                room: room.clone(),
                fen: Some(fen.to_string()),
                trash,
            });
        }

        navigate(&room_path(&room), Default::default());
        show_form.set(Form::None);
    };

    let username_submit = move |e: SubmitEvent| {
        e.prevent_default();
        if let Some(username) = form_data(&e).and_then(|data| data.get("username").as_string()) {
            set_username(&username, chess_board_signals);
        }
        show_form.set(Form::None);
    };

    let options_submit = move |e: SubmitEvent| {
        e.prevent_default();
        let Some(data) = form_data(&e) else {
            return;
        };
        let checked = |name: &str| data.get(name).as_string().is_some_and(|value| value == "on");
        let (validation, sync) = (checked("validation"), checked("sync"));

        chess_board_signals.room_status().update(|status| {
            if let Some(status) = status.as_mut() {
                status.set_options(validation, sync);
            };
        });
        chess_board_signals.send(&ClientMsg::Options { validation, sync });
        show_form.set(Form::None);
    };

    let new_game_submit = move |e: SubmitEvent| {
        e.prevent_default();
        let Some(data) = form_data(&e) else {
            return;
        };
        let color = match data.get("color").as_string().as_deref() {
            Some("white") => SeatChoice::White,
            Some("black") => SeatChoice::Black,
            _ => SeatChoice::Random,
        };
        let time_control = data
            .get("time")
            .as_string()
            .and_then(|time| parse_time_control(&time));

        chess_board_signals.send(&ClientMsg::NewGame {
            color,
            time_control,
        });
        show_form.set(Form::None);
    };

    // a tap next to an optional form closes it
    let close_on_backdrop = move |e: web_sys::MouseEvent| {
        if e.target() == e.current_target() {
            show_form.set(Form::None);
        }
    };

    move || match show_form.get() {
        Form::Join => view! {
            <div class=BACKDROP_CLASS on:click=close_on_backdrop>
                <Join submit=join_submit.clone()/>
            </div>
        }
        .into_any(),
        Form::Username => view! {
            <div class=BACKDROP_CLASS>
                <Username submit=username_submit/>
            </div>
        }
        .into_any(),
        Form::Options => view! {
            <div class=BACKDROP_CLASS on:click=close_on_backdrop>
                <Options chess_board_signals=chess_board_signals submit=options_submit/>
            </div>
        }
        .into_any(),
        Form::NewGame => view! {
            <div class=BACKDROP_CLASS on:click=close_on_backdrop>
                <NewGame submit=new_game_submit/>
            </div>
        }
        .into_any(),
        Form::Invite => view! {
            <div class=BACKDROP_CLASS on:click=close_on_backdrop>
                <Invite chess_board_signals=chess_board_signals show_form=show_form/>
            </div>
        }
        .into_any(),
        Form::None => view! { <div class="hidden"></div> }.into_any(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_path_and_time_control() {
        assert_eq!(room_path(MAIN_ROOM), "/");
        assert_eq!(room_path("abc123"), "/g/abc123");
        assert_eq!(
            parse_time_control("180+2"),
            Some(TimeControl {
                initial_secs: 180,
                increment_secs: 2
            })
        );
        assert_eq!(parse_time_control(""), None);
    }
}
