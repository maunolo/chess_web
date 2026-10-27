use leptos::prelude::*;

/// `initial seconds + increment` values, the server checks them too
const TIME_CONTROLS: [(&str, &str); 7] = [
    ("", "No clock"),
    ("60+0", "1+0 Bullet"),
    ("180+2", "3+2 Blitz"),
    ("300+0", "5+0 Blitz"),
    ("600+0", "10+0 Rapid"),
    ("900+10", "15+10 Rapid"),
    ("1800+0", "30+0 Classical"),
];

#[component]
pub fn NewGame<F>(submit: F) -> impl IntoView
where
    F: Fn(web_sys::SubmitEvent) -> () + 'static,
{
    let color_option = |value: &'static str, label: &'static str| {
        view! {
            <label class="flex items-center gap-1 cursor-pointer">
                <input type="radio" name="color" value=value checked=value == "random"/>
                {label}
            </label>
        }
    };

    view! {
        <form
            class="flex h-fit flex-col justify-center items-center gap-4 bg-white rounded p-4"
            on:submit=submit
        >
            <label class="w-full flex justify-center text-xl">"New game"</label>
            <div class="flex gap-4">
                {color_option("white", "White")}
                {color_option("random", "Random")}
                {color_option("black", "Black")}
            </div>
            <select name="time" class="w-full border border-gray-400 rounded p-2">
                {TIME_CONTROLS
                    .map(|(value, label)| {
                        view! { <option value=value selected=value == "300+0">{label}</option> }
                    })}
            </select>
            <button
                class="w-full border border-gray-400 hover:border-blue-500 hover:text-blue-500 rounded py-2 px-4"
                type="submit"
            >
                "Create game"
            </button>
        </form>
    }
}
