pub mod class_list;
pub mod element_pool;
pub mod elements;
pub mod events;
pub mod jwt;
pub mod style;

use cfg_if::cfg_if;
use serde::{Deserialize, Serialize};

use crate::entities::chess_board::ChessBoardSignals;

#[derive(Serialize, Deserialize, Clone)]
pub struct SessionPayload {
    pub sub: String,
    pub name: String,
    pub iat: u64,
}

cfg_if! {
    if #[cfg(not(feature = "ssr"))] {
        pub fn js_cast<T, C>(to_cast: C) -> Option<T>
            where
            T: wasm_bindgen::JsCast,
            C: wasm_bindgen::JsCast,
        {
            wasm_bindgen::JsCast::dyn_into::<T>(to_cast).ok()
        }
    } else {
        pub fn js_cast<T, C>(_to_cast: C) -> Option<T> {
            None
        }
    }
}

#[allow(unused_variables)]
pub fn get_cookie_value(name: &str) -> Option<String> {
    cfg_if! {
        if #[cfg(not(feature = "ssr"))] {
            let document = web_sys::window()?.document()?;
            let document = js_cast::<web_sys::HtmlDocument, _>(document)?;
            let cookie = document.cookie().ok()?;
            let cookie = cookie
                .split(";")
                .map(|s| s.trim())
                .find(|s| s.starts_with(&format!("{}=", name)))?;
            let cookie = cookie.split("=").nth(1)?;
            Some(cookie.to_string())
        } else {
            None
        }
    }
}
cfg_if! {
    if #[cfg(not(feature = "ssr"))] {
        use wasm_bindgen::JsValue;

        pub fn closure<F>(callback: F) -> wasm_bindgen::prelude::Closure<dyn FnMut()>
        where
            F: FnMut() + 'static,
        {
            wasm_bindgen::prelude::Closure::<dyn FnMut()>::new(callback)
        }

        pub fn closure_with_arg<F>(
            callback: F,
        ) -> wasm_bindgen::prelude::Closure<dyn FnMut(JsValue)>
        where
        F: FnMut(JsValue) + 'static,
        {
            wasm_bindgen::prelude::Closure::<dyn FnMut(JsValue)>::new(callback)
        }

        pub fn closure_with_touch_event<F>(
            callback: F,
        ) -> wasm_bindgen::prelude::Closure<dyn FnMut(web_sys::TouchEvent)>
        where
        F: FnMut(web_sys::TouchEvent) + 'static,
        {
            wasm_bindgen::prelude::Closure::<dyn FnMut(web_sys::TouchEvent)>::new(callback)
        }
    }
}

pub trait WindowExt {
    fn set_timeout_callback<F>(&self, callback: F, miliseconds: i32) -> Option<i32>
    where
        F: FnMut() + 'static;

    fn post_user_name(&self, name: &str, chess_board_signals: ChessBoardSignals);
}

#[allow(unused_variables)]
impl WindowExt for web_sys::Window {
    fn set_timeout_callback<F>(&self, callback: F, miliseconds: i32) -> Option<i32>
    where
        F: FnMut() + 'static,
    {
        cfg_if! {
            if #[cfg(not(feature = "ssr"))] {
                use wasm_bindgen::JsCast;

                let closure = closure(callback);
                let args = js_sys::Array::new();
                let timeout_callback_handle = self.set_timeout_with_callback_and_timeout_and_arguments(
                    closure.as_ref().unchecked_ref(),
                    miliseconds,
                    &args,
                    );
                closure.forget();
                timeout_callback_handle.ok()
            } else {
                None
            }
        }
    }

    fn post_user_name(&self, name: &str, chess_board_signals: ChessBoardSignals) {
        cfg_if! {
            if #[cfg(not(feature = "ssr"))] {
                use wasm_bindgen::JsValue;

                let mut request_init = web_sys::RequestInit::new();
                request_init
                    .method("POST")
                    .body(Some(&JsValue::from_str(&format!("{} \"username\": \"{}\" {}", "{", name, "}"))));
                let promise = self.fetch_with_str_and_init("/sessions", &request_init);

                let start_websocket = closure_with_arg(move |_: JsValue| {
                    chess_board_signals.start_websocket();
                });
                let _ = promise.then(&start_websocket);
                start_websocket.forget();
            }
        }
    }
}
