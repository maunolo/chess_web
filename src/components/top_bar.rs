use leptos::prelude::*;

use crate::{
    components::{forms::Form, icons::Icon, people::PeopleButton},
    entities::chess_board::signals::ChessBoardSignals,
    i18n::{self, Key, Lang},
};

/// What the page is showing: a title and a line under it
pub fn room_title(lang: Lang, chess_board_signals: ChessBoardSignals) -> (String, String) {
    let Some(room) = chess_board_signals
        .room_status()
        .with(|status| status.as_ref().map(|status| status.name()))
    else {
        return ("Chess".to_string(), String::new());
    };

    match chess_board_signals.game().get() {
        Some(game) => {
            let title = match game.time_control {
                Some(time_control) => lang.time_control(&time_control),
                None => lang.t(Key::CasualGame).to_string(),
            };
            (title, lang.game_title(&room))
        }
        None => (lang.room_name(&room), lang.t(Key::FreeBoard).to_string()),
    }
}

#[component]
pub fn TopBar(
    chess_board_signals: ChessBoardSignals,
    show_form: RwSignal<Form>,
    menu_open: RwSignal<bool>,
    people_open: RwSignal<bool>,
) -> impl IntoView {
    let lang = i18n::current();
    let title = move || room_title(lang, chess_board_signals).0;
    let subtitle = move || room_title(lang, chess_board_signals).1;
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
                <button class=menu_btn_class on:click=toggle_menu aria-label=lang.t(Key::Menu)>
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
                    {lang.t(Key::NewGame)}
                </button>
                <button class="btn btn-outline hidden lg:inline-flex h-10" on:click=move |_| show_form.set(Form::Join)>
                    {lang.t(Key::JoinRoom)}
                </button>
                <button class="btn btn-primary hidden lg:inline-flex h-10" on:click=move |_| show_form.set(Form::Invite)>
                    {lang.t(Key::Invite)}
                </button>
                <Show when=move || chess_board_signals.is_game()>
                    <button class="icon-btn lg:hidden" aria-label=lang.t(Key::Invite) on:click=move |_| show_form.set(Form::Invite)>
                        {Icon::Invite.view()}
                    </button>
                </Show>
                <PeopleButton chess_board_signals=chess_board_signals open=people_open menu_open=menu_open/>
            </div>
        </header>
    }
}
