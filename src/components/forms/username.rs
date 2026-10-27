use leptos::prelude::*;

#[component]
pub fn Username<F>(submit: F) -> impl IntoView
where
    F: Fn(web_sys::SubmitEvent) -> () + 'static,
{
    view! {
        <form class="dialog" on:submit=submit>
            <label class="dialog-title" for="username">"What's your name?"</label>
            <p class="text-sm text-ink-300 text-center -mt-2">"Shown to the other players in the room."</p>
            <input
                id="username"
                class="input"
                type="text"
                name="username"
                placeholder="Ex: John Doe"
                autocomplete="nickname"
                maxlength="24"
            />
            <button class="btn btn-primary" type="submit">
                "Continue"
            </button>
        </form>
    }
}
