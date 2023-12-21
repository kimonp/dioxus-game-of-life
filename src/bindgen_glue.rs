//! Glue code to access wasm-bindgen code.

use wasm_bindgen::prelude::*;

pub fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

/// Returns the id of the animation frame.
pub fn request_animation_frame(f: &Closure<dyn FnMut()>) -> i32 {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK")
}

pub fn cancel_animation_frame(animation_id: i32) -> () {
    window().cancel_animation_frame(animation_id).expect("Unable to cancel animation_frame")
}

pub fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}

#[allow(dead_code)]
fn body() -> web_sys::HtmlElement {
    document().body().expect("document should have a body")
}