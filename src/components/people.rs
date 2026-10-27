use leptos::prelude::*;

use crate::{
    components::{forms::Form, icons::Icon},
    entities::{
        chess_board::signals::{ChessBoardSignals, Connection},
        room::{User, UserStatus},
    },
    i18n::{self, Key},
    utils::my_id,
};

/// Everyone in the room, with their seat
#[component]
pub fn PeopleList(chess_board_signals: ChessBoardSignals) -> impl IntoView {
    let lang = i18n::current();
    let users = move || {
        chess_board_signals
            .room_status()
            .with(|status| status.as_ref().map(|s| s.users()).unwrap_or_default())
    };

    let person = move |user: RwSignal<User>| {
        let id = user.with_untracked(|u| u.id());
        let role = {
            let id = id.clone();
            move || {
                let seat = chess_board_signals
                    .game()
                    .with(|game| game.as_ref().and_then(|game| game.seat_of(&id)));
                let seat = match seat {
                    Some(side) => lang.side(side),
                    None if chess_board_signals.is_game() => lang.t(Key::Watching),
                    None => "",
                };
                let me = my_id().as_deref() == Some(id.as_str());
                match (seat, me) {
                    ("", true) => lang.t(Key::You).to_string(),
                    ("", false) => String::new(),
                    (seat, true) => format!("{seat} · {}", lang.t(Key::You)),
                    (seat, false) => seat.to_string(),
                }
            }
        };
        let presence_class = move || match user.with(|u| u.status()) {
            UserStatus::Online => "presence presence--online",
            UserStatus::Away => "presence presence--away",
            UserStatus::Offline => "presence presence--offline",
        };
        let name = move || user.with(|u| u.username());
        let initial = move || {
            name()
                .chars()
                .next()
                .map(|c| c.to_uppercase().to_string())
                .unwrap_or_default()
        };

        view! {
            <li class="person">
                <div aria-hidden="true" class="avatar">{initial}</div>
                <div class="flex-grow min-w-0 flex flex-col">
                    <span class="person-name">{name}</span>
                    <span class="person-role">{role}</span>
                </div>
                <span class=presence_class aria-label=move || user.with(|u| u.status_str())></span>
            </li>
        }
    };

    view! {
        <ul class="flex flex-col gap-0.5">
            <For each=users key=|user| user.with_untracked(|u| u.id()) children=person/>
        </ul>
    }
}

/// The top bar's people count, which opens the list. Shows the connection
/// too: the dot goes red when the socket is down.
#[component]
pub fn PeopleButton(
    chess_board_signals: ChessBoardSignals,
    open: RwSignal<bool>,
    menu_open: RwSignal<bool>,
) -> impl IntoView {
    let lang = i18n::current();
    let count = move || {
        chess_board_signals
            .room_status()
            .with(|status| status.as_ref().map(|s| s.users_count()).unwrap_or(0))
    };
    let online = move || chess_board_signals.connection().get() == Connection::Open;
    let dot_class = move || {
        if online() {
            "w-2 h-2 rounded-full bg-mint"
        } else {
            "w-2 h-2 rounded-full bg-coral animate-pulse-brightness"
        }
    };
    let label = move || lang.in_the_room(count());
    let toggle = move |_| {
        menu_open.set(false);
        open.update(|open| *open = !*open);
    };

    view! {
        <button class="icon-btn w-auto px-2.5 gap-1.5" aria-label=label on:click=toggle>
            {Icon::People.view()}
            <span class="text-[13px] font-semibold">{count}</span>
            <span aria-hidden="true" class=dot_class></span>
        </button>
    }
}

/// The list under the people button, with the name and reconnect controls
#[component]
pub fn PeopleDropdown(
    chess_board_signals: ChessBoardSignals,
    open: RwSignal<bool>,
    show_form: RwSignal<Form>,
) -> impl IntoView {
    let lang = i18n::current();
    let class = move || {
        if open.get() {
            "dropdown dropdown--right dropdown--open"
        } else {
            "dropdown dropdown--right"
        }
    };
    let disconnected = move || chess_board_signals.connection().get() != Connection::Open;
    let reconnect = move |_| {
        chess_board_signals.start_websocket();
        open.set(false);
    };
    let change_name = move |_| {
        open.set(false);
        show_form.set(Form::Username);
    };

    view! {
        <div class=class>
            <span class="eyebrow px-2 pt-1">{lang.t(Key::InTheRoom)}</span>
            <PeopleList chess_board_signals=chess_board_signals/>
            <div class="border-t border-ink-700 mt-1 pt-1 flex flex-col gap-1">
                <button class="dropdown-item" on:click=change_name>
                    {Icon::Name.view()}
                    {lang.t(Key::ChangeName)}
                </button>
                <Show when=disconnected>
                    <button class="dropdown-item text-coral" on:click=reconnect>
                        {Icon::Refresh.view()}
                        {lang.t(Key::Reconnect)}
                    </button>
                </Show>
            </div>
        </div>
    }
}
