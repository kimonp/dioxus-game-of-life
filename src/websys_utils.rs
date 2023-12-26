//! Short cuts for websys functions.

use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (
        web_sys::console::log_1(&format!($($t)*).into())
    )
}

pub fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

/// Returns the id of the animation frame.
pub fn request_animation_frame(f: &Closure<dyn FnMut()>) -> i32 {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK")
}

/// Cancel a running aninmation frame.
pub fn cancel_animation_frame(animation_id: i32) {
    window().cancel_animation_frame(animation_id).expect("Unable to cancel animation_frame")
}

/// Get the DOM document.
pub fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}

/// Return the 2d canvas context of the given element id.
pub fn get_2d_context(element_id: &str) -> CanvasRenderingContext2d {
    let canvas_ele = document().get_element_by_id(element_id).expect("requested element not found");
    let canvas_ele: web_sys::HtmlCanvasElement = canvas_ele
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    canvas_ele
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap()
}
