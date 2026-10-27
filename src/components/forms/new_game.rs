use leptos::prelude::*;

/// `initial seconds + increment` values, the server checks them too
const TIME_CONTROLS: [(&str, &str); 7] = [
    ("", "No clock"),
    ("60+0", "1+0 · Bullet"),
    ("180+2", "3+2 · Blitz"),
    ("300+0", "5+0 · Blitz"),
    ("600+0", "10+0 · Rapid"),
    ("900+10", "15+10 · Rapid"),
    ("1800+0", "30+0 · Classical"),
];

#[component]
pub fn NewGame<F>(submit: F) -> impl IntoView
where
    F: Fn(web_sys::SubmitEvent) -> () + 'static,
{
    let color_option = |value: &'static str, label: &'static str| {
        view! {
            <label class="flex-1 cursor-pointer">
                <input type="radio" name="color" value=value checked=value == "random" class="peer sr-only"/>
                <span class="btn btn-secondary w-full peer-checked:bg-accent peer-checked:text-ink-900 peer-focus-visible:ring-2 peer-focus-visible:ring-accent">
                    {label}
                </span>
            </label>
        }
    };

    view! {
        <form class="dialog" on:submit=submit>
            <span class="dialog-title">"New game"</span>
            <div class="flex flex-col gap-2">
                <span class="eyebrow">"Play as"</span>
                <div class="flex gap-2">
                    {color_option("white", "White")}
                    {color_option("random", "Random")}
                    {color_option("black", "Black")}
                </div>
            </div>
            <div class="flex flex-col gap-2">
                <label class="eyebrow" for="new-game-time">"Clock"</label>
                <select id="new-game-time" name="time" class="input appearance-none">
                    {TIME_CONTROLS
                        .map(|(value, label)| {
                            view! { <option value=value selected=value == "300+0">{label}</option> }
                        })}
                </select>
            </div>
            <button class="btn btn-primary" type="submit">
                "Create game"
            </button>
        </form>
    }
}
