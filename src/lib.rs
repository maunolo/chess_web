mod app;
mod components;
mod entities;

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "hydrate")] {
        mod handlers;
        mod utils;
        mod client;

        use wasm_bindgen::prelude::wasm_bindgen;
        use crate::app::*;
        use leptos::*;

        #[wasm_bindgen]
        pub fn hydrate() {
            _ = console_log::init_with_level(log::Level::Debug);
            console_error_panic_hook::set_once();

            log!("hydrate mode - hydrating");

            leptos::mount_to_body(|cx| {
                view! { cx,  <App/> }
            });
        }
    }
    else if #[cfg(feature = "csr")] {
        mod handlers;
        mod utils;
        mod client;

        use wasm_bindgen::prelude::wasm_bindgen;

        #[wasm_bindgen(start)]
        pub fn main() {
            use app::*;
            use leptos::*;
            _ = console_log::init_with_level(log::Level::Debug);
            console_error_panic_hook::set_once();

            log!("csr mode - mounting to body");

            mount_to_body(|cx| {
                view! { cx, <App /> }
            });
        }
    }
}
