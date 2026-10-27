use leptos::prelude::*;

#[component]
pub fn Join<F>(submit: F) -> impl IntoView
where
    F: Fn(web_sys::SubmitEvent) -> () + 'static,
{
    view! {
        <form class="dialog" on:submit=submit>
            <label class="dialog-title" for="join-room">"Join a room"</label>
            <p class="text-sm text-ink-300 text-center -mt-2">
                "A game's code, or any name for a free board. A FEN after the name starts a new free board from that position."
            </p>
            <input
                id="join-room"
                class="input"
                type="text"
                name="room"
                placeholder="Room name or game code"
                autocomplete="off"
            />
            <button class="btn btn-primary" type="submit">
                "Join"
            </button>
        </form>
    }
}
