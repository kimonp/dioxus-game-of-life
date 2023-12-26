//! Animation frame control via a custom Dioxus hook.

use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;
use wasm_bindgen::prelude::Closure;

use crate::websys_utils::*;

/// A custom Dioxus hook that abstracts the request_animation_frame() and cancel_animation_frame() DOM calls.
///
/// Allows the caller to create a use_effect which watches the frame_id which can then
/// take an action each time a frame is advanced.
///
/// Returns two UseState variables: frame_running and frame_id.
/// * frame_running is true if frames are advancing.
/// * frame_id is incremented each time a new frame is run.
///
/// If frame_running is set to true, frames advance.
/// If frame_running is set to false, frames stop advancing.
pub fn use_animation_frame(cx: Scope, initial_state: bool) -> (&UseState<bool>, &UseState<i32>) {
    let frame_running = use_state(cx, || initial_state);
    let cancel_id = use_state(cx, || None::<i32>);
    let frame_id = use_state(cx, || 0_i32);

    use_effect(cx, (frame_running,), |(frame_running,)| {
        to_owned![cancel_id, frame_id, frame_running];

        // frame_loop_holder holds a closure that is passed to request_animation_frame().
        // This closure is called each time an animation frame completes.  We modify the universe
        // inside this closure.
        let frame_loop_holder = Rc::new(RefCell::new(None));
        let frame_loop_holder_clone = frame_loop_holder.clone();

        let cancel_id_clone = cancel_id.clone();
        *frame_loop_holder.borrow_mut() = Some(Closure::<dyn FnMut()>::new(move || {
            let new_id =
                request_animation_frame(frame_loop_holder_clone.borrow().as_ref().unwrap());
            cancel_id_clone.set(Some(new_id));

            frame_id.with_mut(|id| {
                *id = id.wrapping_add(1);
            })
        }));

        async move {
            // If we are requested to run, but we are not running, run
            if *frame_running.get() && cancel_id.get().is_none() {
                let new_id = request_animation_frame(frame_loop_holder.borrow().as_ref().unwrap());
                cancel_id.set(Some(new_id));
            }

            // If we are requested to stop, but we are running, cancel
            if !*frame_running.get() && cancel_id.get().is_some() {
                cancel_id.with_mut(|maybe_id| {
                    if let Some(id) = maybe_id {
                        cancel_animation_frame(*id);
                        *maybe_id = None;
                    }
                });
            }
        }
    });

    (frame_running, frame_id)
}
