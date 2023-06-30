pub mod class_list;
pub mod element_pool;
pub mod elements;
pub mod style;

pub trait WindowExt {
    fn set_timeout_callback<F>(&self, callback: F, miliseconds: i32) -> Option<i32>
    where
        F: FnMut() + 'static;
}

impl WindowExt for web_sys::Window {
    fn set_timeout_callback<F>(&self, callback: F, miliseconds: i32) -> Option<i32>
    where
        F: FnMut() + 'static,
    {
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
    }
}
