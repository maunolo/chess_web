use leptos::*;

fn row_str(y: isize) -> String {
    (8 - y).to_string()
}

fn col_str(x: isize) -> String {
    let buffer: [u8; 1] = [x as u8 + 97];
    std::str::from_utf8(&buffer).unwrap().to_string()
}

#[component]
pub fn Coordinates<F>(white_view: F) -> impl IntoView
where
    F: Fn() -> bool + 'static,
{
    let positions = move || {
        if white_view() {
            (0..8).collect::<Vec<_>>()
        } else {
            (0..8).rev().collect::<Vec<_>>()
        }
    };

    view! {
        <div class="coordinates">
            <For
                each=positions
                key=|i| i.clone()
                children=move |pos: isize| {
                    view! {
                        <span class=format!("pointer-events-none absolute leading-3 opacity-60 text-xs coord-row-{}", pos)>
                            {row_str(pos)}
                        </span>
                        <span class=format!("pointer-events-none absolute leading-3 opacity-60 text-xs coord-col-{}", pos)>
                            {col_str(pos)}
                        </span>
                    }
                }
            />
        </div>
    }
}
