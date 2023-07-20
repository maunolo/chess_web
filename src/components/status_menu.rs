use leptos::*;

use crate::{
    components::overlay::{clear_timeout, get_user_payload, toggle_sub_menu, Form},
    entities::{chess_board::signals::ChessBoardSignals, room::User},
};

#[component]
pub fn StatusMenu(
    cx: Scope,
    show_form: RwSignal<Form>,
    chess_board_signals: ChessBoardSignals,
) -> impl IntoView {
    let show_status_menu = create_rw_signal(cx, false);
    let status_menu_timeout_id = create_rw_signal::<Option<i32>>(cx, None);

    let username = move |_| {
        show_form.set(Form::Username);
    };

    let toggle_status_menu = move |_| {
        show_status_menu.update(|show_status_menu| {
            clear_timeout(status_menu_timeout_id.get());
            toggle_sub_menu(
                status_menu_timeout_id,
                !*show_status_menu,
                "status-sub-menu",
            );

            *show_status_menu = !*show_status_menu;
        });
    };

    let start_websocket = move |_| {
        chess_board_signals.start_websocket();
    };

    let status_menu_css = move || {
        let is_active = show_status_menu.get();
        let mut class = "status-menu".to_string();

        if is_active {
            class.push_str(" status-menu--is-active");
        }

        class
    };

    let status_menu_btn_css = move || {
        let is_active = show_status_menu.get();
        let is_online = chess_board_signals.socket().with(|s| s.is_some());
        let mut class = "status-menu-btn".to_string();

        if is_online && is_active {
            class.push_str(" status-menu-btn--is-online--is-active");
        } else if is_online {
            class.push_str(" status-menu-btn--is-online");
        } else if is_active {
            class.push_str(" status-menu-btn--is-active");
        }

        class
    };

    let status_refresh_btn_css = move || {
        let needs_refresh = chess_board_signals.socket().get().is_none();
        let mut class = "status-refresh-btn".to_string();

        if needs_refresh {
            class.push_str(" status-refresh-btn--is-active");
        }

        class
    };

    let status_menu_style = move || {
        if show_status_menu.get() {
            let mut users_count = chess_board_signals
                .room_status()
                .with(|status| status.as_ref().map(|s| s.users_count()).unwrap_or(0));
            if users_count > 12 {
                users_count = 12;
            } else if users_count < 1 {
                users_count = 1;
            }
            let mut height = 3.75 + (2 * (users_count - 1)) as f64;
            if users_count > 1 {
                height += 0.5;
            }

            format!("height: {}rem;", height)
        } else {
            "".to_string()
        }
    };

    let users = move || {
        chess_board_signals
            .room_status()
            .with(|status| status.as_ref().map(|s| s.users()).unwrap_or(vec![]))
    };

    let user_view = move |cx, user: RwSignal<User>| {
        let status_class = move || format!("status status--{}", user.with(|u| u.status_str()));
        if user.with(|u| u.id()) == get_user_payload().map(|p| p.sub).unwrap_or_default() {
            view! {
                cx,
                <li class="current-user">
                    <span>
                        {move || user.with(|u| u.username())}
                    </span>
                    <button on:click=username>
                        <svg xmlns="http://www.w3.org/2000/svg" id="Layer_1" data-name="Layer 1" viewBox="0 0 24 24" width="512" height="512">
                            <polygon points="14.604 5.687 0 20.29 0 24 3.71 24 18.313 9.396 14.604 5.687"/>
                            <path d="M23.232.768a2.624,2.624,0,0,0-3.71,0l-3.5,3.505,3.709,3.709,3.5-3.5A2.624,2.624,0,0,0,23.232.768Z"/>
                        </svg>
                    </button>
                </li>
            }
        } else {
            view! {
                cx,
                <li>
                    <span>
                        {move || user.with(|u| u.username())}
                    </span>
                    <span class=status_class>
                    </span>
                </li>
            }
        }
    };

    view! { cx,
        <div class=status_menu_css style=status_menu_style>
            <div class="status-menu-header">
                <button
                    class=status_menu_btn_css
                    on:click=toggle_status_menu
                >
                    <span class="circle">
                        <span class="circle-inner">{move || users().len()}</span>
                    </span>
                </button>
                <button
                    class=status_refresh_btn_css
                    on:click=start_websocket
                >
                    <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" version="1.1" id="Capa_1" x="0px" y="0px" viewBox="0 0 513.806 513.806" style="enable-background:new 0 0 513.806 513.806;" xml:space="preserve" width="512" height="512">
                        <g>
                            <path d="M66.074,228.731C81.577,123.379,179.549,50.542,284.901,66.045c35.944,5.289,69.662,20.626,97.27,44.244l-24.853,24.853   c-8.33,8.332-8.328,21.84,0.005,30.17c3.999,3.998,9.423,6.245,15.078,6.246h97.835c11.782,0,21.333-9.551,21.333-21.333V52.39   c-0.003-11.782-9.556-21.331-21.338-21.329c-5.655,0.001-11.079,2.248-15.078,6.246L427.418,65.04   C321.658-29.235,159.497-19.925,65.222,85.835c-33.399,37.467-55.073,83.909-62.337,133.573   c-2.864,17.607,9.087,34.202,26.693,37.066c1.586,0.258,3.188,0.397,4.795,0.417C50.481,256.717,64.002,244.706,66.074,228.731z"/>
                            <path d="M479.429,256.891c-16.108,0.174-29.629,12.185-31.701,28.16C432.225,390.403,334.253,463.24,228.901,447.738   c-35.944-5.289-69.662-20.626-97.27-44.244l24.853-24.853c8.33-8.332,8.328-21.84-0.005-30.17   c-3.999-3.998-9.423-6.245-15.078-6.246H43.568c-11.782,0-21.333,9.551-21.333,21.333v97.835   c0.003,11.782,9.556,21.331,21.338,21.329c5.655-0.001,11.079-2.248,15.078-6.246l27.733-27.733   c105.735,94.285,267.884,85.004,362.17-20.732c33.417-37.475,55.101-83.933,62.363-133.615   c2.876-17.605-9.064-34.208-26.668-37.084C482.655,257.051,481.044,256.91,479.429,256.891z"/>
                        </g>
                    </svg>
                </button>
            </div>
            <div class="status-sub-menu" id="status-sub-menu">
                <ul>
                    <For
                        each=users
                        key=|user| user.with(|u| u.id())
                        view=user_view
                    />
                </ul>
            </div>
        </div>
    }
}
