pub mod app;
mod components;
pub mod entities;
mod handlers;
pub mod utils;

#[cfg(feature = "ssr")]
pub mod server;

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "hydrate")] {
        mod client;

        use wasm_bindgen::prelude::wasm_bindgen;
        use crate::app::App;

        #[wasm_bindgen]
        pub fn hydrate() {
            _ = console_log::init_with_level(log::Level::Debug);
            console_error_panic_hook::set_once();

            leptos::mount::hydrate_body(App);
        }
    }
}
