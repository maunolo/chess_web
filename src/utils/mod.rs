pub mod class_list;
pub mod element_pool;
pub mod elements;
pub mod events;
pub mod style;

use cfg_if::cfg_if;

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

pub trait WindowExt {
    fn set_timeout_callback<F>(&self, callback: F, miliseconds: i32) -> Option<i32>
    where
        F: FnMut() + 'static;
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
                let closure = wasm_bindgen::prelude::Closure::<dyn FnMut()>::new(callback);
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
}
