use leptos::*;

fn row_str(y: isize) -> String {
    (8 - y).to_string()
}

fn col_str(x: isize) -> String {
    let buffer: [u8; 1] = [x as u8 + 97];
    std::str::from_utf8(&buffer).unwrap().to_string()
}

#[component]
pub fn Coordinates<F>(cx: Scope, white_view: F) -> impl IntoView
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
      cx,
      <div class="coordinates">
        <For
          // a function that returns the items we're iterating over; a signal is fine
          each=positions
          // a unique key for each item
          key=|i| i.clone()
          // renders each item to a view
          view=move |cx, pos: isize| {
            view! {
              cx,
            //   <span x="0.50" y={format!("{}", 12.5 * i as f64 + 2.25)}>{row_str(pos)}</span>
            //   <span x={format!("{}", 12.5 * i as f64 + 10.75)} y="99.50">{col_str(pos)}</span>
              <span class={format!("pointer-events-none absolute leading-3 opacity-60 text-xs coord-row-{}", pos)}>{row_str(pos)}</span>
              <span class={format!("pointer-events-none absolute leading-3 opacity-60 text-xs coord-col-{}", pos)}>{col_str(pos)}</span>
            }
          }
        />
      </div>
    }
}
