mod app;
mod components;
mod entities;
mod handlers;
mod utils;

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "hydrate")] {
        mod client;

        use wasm_bindgen::prelude::wasm_bindgen;
        use crate::app::*;
        use leptos::*;

        #[wasm_bindgen]
        pub fn hydrate() {
            _ = console_log::init_with_level(log::Level::Debug);
            console_error_panic_hook::set_once();

            leptos::mount_to_body(|| {
                view! { <App/> }
            });
        }
    }
    else if #[cfg(feature = "csr")] {
        mod client;

        use wasm_bindgen::prelude::wasm_bindgen;

        #[wasm_bindgen(start)]
        pub fn main() {
            use app::*;
            use leptos::*;
            _ = console_log::init_with_level(log::Level::Debug);
            console_error_panic_hook::set_once();

            mount_to_body(|| {
                view! { <App /> }
            });
        }
    }
}
