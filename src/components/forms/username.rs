use leptos::prelude::*;

use crate::i18n::{self, Key};

#[component]
pub fn Username<F>(submit: F) -> impl IntoView
where
    F: Fn(web_sys::SubmitEvent) -> () + 'static,
{
    let lang = i18n::current();

    view! {
        <form class="dialog" on:submit=submit>
            <label class="dialog-title" for="username">{lang.t(Key::WhatsYourName)}</label>
            <p class="text-sm text-ink-300 text-center -mt-2">{lang.t(Key::NameHint)}</p>
            <input
                id="username"
                class="input"
                type="text"
                name="username"
                placeholder=lang.t(Key::NamePlaceholder)
                autocomplete="nickname"
                maxlength="24"
            />
            <button class="btn btn-primary" type="submit">
                {lang.t(Key::Continue)}
            </button>
        </form>
    }
}
