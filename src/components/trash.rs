use leptos::*;

use crate::entities::stone::Stone;

use cfg_if::cfg_if;

#[derive(Clone, Copy, PartialEq)]
pub enum TrashType {
    Dark,
    Light,
}

impl From<TrashType> for String {
    fn from(trash_type: TrashType) -> Self {
        match trash_type {
            TrashType::Dark => "dark".to_string(),
            TrashType::Light => "light".to_string(),
        }
    }
}

#[component]
pub fn Trash<W, T>(cx: Scope, id: TrashType, white_view: W, trash: T) -> impl IntoView
where
    W: Fn() -> bool + 'static,
    T: Fn() -> Vec<Stone> + 'static,
{
    cfg_if! {
      if #[cfg(feature = "ssr")] {
        let mousedown = move |_| {};
        let touchstart = move |_| {};
      } else {
        use crate::handlers::mouse::{mousedown, touchstart};
      }
    }

    let position_css = move || {
        if (matches!(id, TrashType::Dark) && white_view())
            || (matches!(id, TrashType::Light) && !white_view())
        {
            "-bottom-12 rounded-b".to_string()
        } else {
            "-top-12 rounded-t".to_string()
        }
    };
    let trash = move || {
        trash()
            .into_iter()
            .filter(move |stone| stone.color.to_lowercase() == String::from(id))
            .enumerate()
    };
    let trash_class = move || {
        format!(
            "flex absolute h-12 w-full z-20 bg-neutral-500 {}",
            position_css()
        )
    };
    let trash_id = move || format!("{}-trash", String::from(id));
    view! { cx,
        <div class=trash_class data-trash=move || String::from(id) id=trash_id>
            <For
                each=trash
                key=move |(idx, stone)| { format!("{}-{}", idx, stone.image_class) }
                view=move |cx, (_, stone)| {
                    view! { cx,
                        <div
                            class=format!("piece {} deleted", stone.image_class.clone())
                            on:mousedown=mousedown
                            on:touchstart=touchstart
                            on:dragstart=move |e| e.prevent_default()
                            data-square="deleted"
                            data-piece=stone.image_class.clone()
                        ></div>
                    }
                }
            />
        </div>
    }
}
