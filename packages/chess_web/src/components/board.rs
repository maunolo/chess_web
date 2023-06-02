use leptos::*;

#[component]
pub fn BoardBackground(cx: Scope) -> impl IntoView {
    view! {
      cx,
      <svg viewBox="0 0 100 100" class="board">
        <For
          // a function that returns the items we're iterating over; a signal is fine
          each={move || (0..32)}
          // a unique key for each item
          key=|i| i.clone()
          // renders each item to a view
          view=move |cx, i: usize| {
            let x = 12.5 * (i % 8) as f64;
            let y = 25.0 * (i / 8) as f64;
            let rotate = format!("rotate(180, {}, {})", x + 6.25, y + 12.5);
            view! {
              cx,
              <image
                href="/static/chess/board/aluminium.png"
                x={format!("{}", x)}
                y={format!("{}", y)}
                height="25"
                width="12.5"
                transform={if i % 2 == 0 {"".to_string()} else {rotate}}
              />
            }
          }
        />
      </svg>
    }
}
