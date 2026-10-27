use leptos::prelude::*;

use crate::{
    components::forms::{room_path, Form},
    entities::chess_board::signals::ChessBoardSignals,
    utils::browser::{can_share, copy_to_clipboard, page_origin, share_url},
};

const BUTTON_CLASS: &str =
    "flex-1 border border-gray-400 hover:border-blue-500 hover:text-blue-500 rounded py-2 px-3";

#[component]
pub fn Invite(chess_board_signals: ChessBoardSignals, show_form: RwSignal<Form>) -> impl IntoView {
    let link = move || {
        let room = chess_board_signals
            .room_status()
            .with(|status| status.as_ref().map(|status| status.name()))
            .unwrap_or_default();
        format!("{}{}", page_origin(), room_path(&room))
    };
    let copied = RwSignal::new(false);

    view! {
        <div class="flex flex-col gap-3 bg-white rounded p-4 w-80 max-w-full">
            <label class="text-xl text-center">"Invite your opponent"</label>
            <p class="text-sm text-center text-neutral-600">
                "Whoever opens this link first takes the free seat."
            </p>
            <input class="border border-gray-400 rounded p-2 text-sm" readonly prop:value=link/>
            <div class="flex gap-2">
                <button class=BUTTON_CLASS on:click=move |_| copied.set(copy_to_clipboard(&link()))>
                    {move || if copied.get() { "Copied!" } else { "Copy link" }}
                </button>
                {can_share()
                    .then(|| {
                        view! {
                            <button
                                class=BUTTON_CLASS
                                on:click=move |_| {
                                    share_url("Play chess with me", &link());
                                }
                            >
                                "Share"
                            </button>
                        }
                    })}
                <button class=BUTTON_CLASS on:click=move |_| show_form.set(Form::None)>
                    "Close"
                </button>
            </div>
        </div>
    }
}
