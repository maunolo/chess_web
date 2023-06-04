use leptos::*;

fn row_str(y: isize) -> String {
    (8 - y).to_string()
}

fn col_str(x: isize) -> String {
    let buffer: [u8; 1] = [x as u8 + 97];
    std::str::from_utf8(&buffer).unwrap().to_string()
}

#[component]
pub fn Coordinates(cx: Scope, is_white_view: bool) -> impl IntoView {
    view! {
      cx,
      <svg viewBox="0 0 100 100" class="coordinates">
        <For
          // a function that returns the items we're iterating over; a signal is fine
          each={move || (0..8)}
          // a unique key for each item
          key=|i| i.clone()
          // renders each item to a view
          view=move |cx, i: isize| {
            let y = if is_white_view { i } else { 7 - i };
            view! {
              cx,
              <text x="0.50" y={format!("{}", 12.5 * i as f64 + 2.25)} font-size="2">{row_str(y)}</text>
            }
          }
        />
        <For
          // a function that returns the items we're iterating over; a signal is fine
          each={move || (0..8)}
          // a unique key for each item
          key=|i| i.clone()
          // renders each item to a view
          view=move |cx, i: isize| {
            let x = if is_white_view { i } else { 7 - i };
            view! {
              cx,
              <text x={format!("{}", 12.5 * i as f64 + 10.75)} y="99.50" font-size="2">{col_str(x)}</text>
            }
          }
        />
      </svg>
    }
}
