use leptos::prelude::*;

use crate::{
    components::forms::{room_path, Form},
    entities::chess_board::signals::ChessBoardSignals,
    utils::browser::{can_share, copy_to_clipboard, page_origin, share_url},
};

#[component]
pub fn Invite(chess_board_signals: ChessBoardSignals, show_form: RwSignal<Form>) -> impl IntoView {
    let link = move || {
        let room = chess_board_signals
            .room_status()
            .with(|status| status.as_ref().map(|status| status.name()))
            .unwrap_or_default();
        format!("{}{}", page_origin(), room_path(&room))
    };
    let hint = move || {
        if chess_board_signals.is_game() {
            "Whoever opens this link first takes the free seat; everyone after that watches."
        } else {
            "Anyone with this link joins the board."
        }
    };
    let copied = RwSignal::new(false);

    view! {
        <div class="dialog">
            <span class="dialog-title">"Invite"</span>
            <p class="text-sm text-ink-300 text-center -mt-2">{hint}</p>
            <input class="input font-mono text-[13px]" readonly prop:value=link/>
            <div class="flex gap-2">
                <button class="btn btn-primary flex-1" on:click=move |_| copied.set(copy_to_clipboard(&link()))>
                    {move || if copied.get() { "Copied!" } else { "Copy link" }}
                </button>
                {can_share()
                    .then(|| {
                        view! {
                            <button
                                class="btn btn-secondary flex-1"
                                on:click=move |_| {
                                    share_url("Play chess with me", &link());
                                }
                            >
                                "Share"
                            </button>
                        }
                    })}
            </div>
            <button class="btn btn-ghost" on:click=move |_| show_form.set(Form::None)>
                "Close"
            </button>
        </div>
    }
}
