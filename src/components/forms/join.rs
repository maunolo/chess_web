use leptos::prelude::*;

use crate::i18n::{self, Key};

#[component]
pub fn Join<F>(submit: F) -> impl IntoView
where
    F: Fn(web_sys::SubmitEvent) -> () + 'static,
{
    let lang = i18n::current();

    view! {
        <form class="dialog" on:submit=submit>
            <label class="dialog-title" for="join-room">{lang.t(Key::JoinARoom)}</label>
            <p class="text-sm text-ink-300 text-center -mt-2">{lang.t(Key::JoinHint)}</p>
            <input
                id="join-room"
                class="input"
                type="text"
                name="room"
                placeholder=lang.t(Key::JoinPlaceholder)
                autocomplete="off"
            />
            <button class="btn btn-primary" type="submit">
                {lang.t(Key::Join)}
            </button>
        </form>
    }
}
