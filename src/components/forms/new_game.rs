use leptos::prelude::*;

use crate::{
    components::forms::parse_time_control,
    i18n::{self, Key},
};

/// `initial seconds + increment` values, the server checks them too
const TIME_CONTROLS: [&str; 7] = ["", "60+0", "180+2", "300+0", "600+0", "900+10", "1800+0"];

#[component]
pub fn NewGame<F>(submit: F) -> impl IntoView
where
    F: Fn(web_sys::SubmitEvent) -> () + 'static,
{
    let lang = i18n::current();
    let color_option = |value: &'static str, label: Key| {
        view! {
            <label class="flex-1 cursor-pointer">
                <input type="radio" name="color" value=value checked=value == "random" class="peer sr-only"/>
                <span class="btn btn-secondary w-full peer-checked:bg-accent peer-checked:text-ink-900 peer-focus-visible:ring-2 peer-focus-visible:ring-accent">
                    {lang.t(label)}
                </span>
            </label>
        }
    };
    let time_label = move |value: &str| match parse_time_control(value) {
        Some(time_control) => lang.time_control(&time_control),
        None => lang.t(Key::NoClock).to_string(),
    };

    view! {
        <form class="dialog" on:submit=submit>
            <span class="dialog-title">{lang.t(Key::NewGame)}</span>
            <div class="flex flex-col gap-2">
                <span class="eyebrow">{lang.t(Key::PlayAs)}</span>
                <div class="flex gap-2">
                    {color_option("white", Key::White)}
                    {color_option("random", Key::Random)}
                    {color_option("black", Key::Black)}
                </div>
            </div>
            <div class="flex flex-col gap-2">
                <label class="eyebrow" for="new-game-time">{lang.t(Key::Clock)}</label>
                <select id="new-game-time" name="time" class="input appearance-none">
                    {TIME_CONTROLS
                        .map(|value| {
                            view! { <option value=value selected=value == "300+0">{time_label(value)}</option> }
                        })}
                </select>
            </div>
            <button class="btn btn-primary" type="submit">
                {lang.t(Key::CreateGame)}
            </button>
        </form>
    }
}
