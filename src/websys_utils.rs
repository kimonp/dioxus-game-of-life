//! Short cuts for websys functions.

use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, Element};

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
    window()
        .cancel_animation_frame(animation_id)
        .expect("Unable to cancel animation_frame")
}

pub fn into_canvas_element(element: &Element) -> web_sys::HtmlCanvasElement {
    element
        .clone()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap()
}

/// Extract 2d rendering context from a canvas element.
pub fn into_2d_context(canvas_ele: &HtmlCanvasElement) -> CanvasRenderingContext2d {
    canvas_ele
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap()
}
