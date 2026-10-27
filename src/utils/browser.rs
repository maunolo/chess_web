//! Browser APIs the UI calls from event handlers and effects. They do nothing
//! in the server build, where those never run.

#[cfg(feature = "hydrate")]
use wasm_bindgen::{JsCast, JsValue};

/// Milliseconds since the epoch
pub fn now_ms() -> f64 {
    #[cfg(feature = "hydrate")]
    return js_sys::Date::now();
    #[cfg(not(feature = "hydrate"))]
    return 0.0;
}

/// `https://host:port` of the page
pub fn page_origin() -> String {
    web_sys::window()
        .and_then(|window| window.location().origin().ok())
        .unwrap_or_default()
}

/// Calls `navigator[object][method](arg)`, or `navigator[method](arg)` without
/// an object. False when the browser doesn't have it.
#[cfg(feature = "hydrate")]
fn call_navigator(object: Option<&str>, method: &str, arg: &JsValue) -> bool {
    let Some(navigator) = web_sys::window().map(|window| JsValue::from(window.navigator())) else {
        return false;
    };
    let target = match object {
        Some(object) => match js_sys::Reflect::get(&navigator, &object.into()) {
            Ok(target) if !target.is_undefined() => target,
            _ => return false,
        },
        None => navigator,
    };

    js_sys::Reflect::get(&target, &method.into())
        .ok()
        .and_then(|function| function.dyn_into::<js_sys::Function>().ok())
        .is_some_and(|function| function.call1(&target, arg).is_ok())
}

/// False when the browser doesn't allow it (it needs https or localhost)
#[allow(unused_variables)]
pub fn copy_to_clipboard(text: &str) -> bool {
    #[cfg(feature = "hydrate")]
    return call_navigator(Some("clipboard"), "writeText", &text.into());
    #[cfg(not(feature = "hydrate"))]
    return false;
}

/// The share sheet of phones and some desktop browsers
pub fn can_share() -> bool {
    #[cfg(feature = "hydrate")]
    return web_sys::window().is_some_and(|window| {
        js_sys::Reflect::has(&window.navigator(), &"share".into()).unwrap_or(false)
    });
    #[cfg(not(feature = "hydrate"))]
    return false;
}

#[allow(unused_variables)]
pub fn share_url(title: &str, url: &str) -> bool {
    #[cfg(feature = "hydrate")]
    {
        let data = js_sys::Object::new();
        let _ = js_sys::Reflect::set(&data, &"title".into(), &title.into());
        let _ = js_sys::Reflect::set(&data, &"url".into(), &url.into());
        return call_navigator(None, "share", &data);
    }
    #[cfg(not(feature = "hydrate"))]
    return false;
}

#[allow(unused_variables)]
pub fn expire_cookie(name: &str) {
    #[cfg(feature = "hydrate")]
    if let Some(document) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.dyn_into::<web_sys::HtmlDocument>().ok())
    {
        let _ = document.set_cookie(&format!("{name}=; Max-Age=0; path=/"));
    }
}
