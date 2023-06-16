use leptos::*;

use crate::entities::{chess_board::ChessBoard, stone::Stone};

use cfg_if::cfg_if;

#[component]
pub fn Trash<F>(
    cx: Scope,
    id: String,
    white_view: F,
    chessboard: ReadSignal<ChessBoard>,
) -> impl IntoView
where
    F: Fn() -> bool + 'static,
{
    cfg_if! {
      if #[cfg(feature = "ssr")] {
        let mousedown = move |_| {};
        let touchstart = move |_| {};
      } else {
        use crate::handlers::mouse::{mousedown, touchstart};
      }
    }

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
        <div class={move || format!("flex absolute h-12 w-full z-20 bg-neutral-500 {}", position())} data-trash={id.clone()} id={move || format!("{}-trash", id_clone)} >
            <For
                // a function that returns the items we're iterating over; a signal is fine
                each=move || {
                    let id_clone = id.clone();
                    chessboard.with(move |cb| {
                        cb
                            .deleted_stones()
                            .clone()
                            .into_iter()
                            .filter(move |s| s.color.to_lowercase() == id_clone)
                            .enumerate()
                    })
                }
                // a unique key for each item
                key=move |(idx, stone)| {
                    format!("{}-{}", idx, stone.image_class)
                }
                // renders each item to a view
                view=move |cx, (_, stone)| {
                    view! {
                        cx,
                        <div
                            class={format!("piece {} deleted", stone.image_class.clone())}
                            on:mousedown=move |e| mousedown(e)
                            on:touchstart=move |e| touchstart(e)
                            on:dragstart=move |e| e.prevent_default()
                            data-square="deleted"
                            data-piece={stone.image_class.clone()}
                        ></div>
                    }
                }
            />
        </div>
    }
}
