use leptos::*;

#[component]
pub fn Trash<F>(cx: Scope, id: String, white_view: F) -> impl IntoView
where
    F: Fn() -> bool + 'static,
{
    let id_clone = id.clone();
    let position = move || {
        let is_white_view = white_view();
        if (id_clone == "dark" && is_white_view) || (id_clone == "light" && !is_white_view) {
            "-bottom-12".to_string()
        } else {
            "-top-12".to_string()
        }
    };

    let id_clone = id.clone();
    view! {
        cx,
        <div class={move || format!("flex absolute h-12 w-full z-20 bg-neutral-500 {}", position())} data-trash={id} id={move || format!("{}-trash", id_clone)} >
        </div>
    }
}
