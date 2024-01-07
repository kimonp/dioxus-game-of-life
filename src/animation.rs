//! Animation frame control via a custom Dioxus hook: use_animation_frame()

use dioxus::prelude::*;

/// A custom Dioxus hook that abstracts the request_animation_frame() and cancel_animation_frame() DOM calls.
///
/// Allows the caller to create a use_effect() which watches the frame_id,
/// which can then take an action each time a frame is advanced.
///
/// Returns two UseState variables: frame_running and frame_id.
/// * frame_running is true if frames are advancing.
/// * frame_id is incremented each time a new frame is run.
///
/// If frame_running is set to true, frames advance.
/// If frame_running is set to false, frames stop advancing.
#[cfg(feature = "web")]
pub fn use_animation_frame(cx: Scope, initial_state: bool) -> (&UseState<bool>, &UseState<i32>) {
    use std::cell::RefCell;
    use std::rc::Rc;

    use wasm_bindgen::prelude::Closure;

    use crate::websys_utils::*;

    let frame_running = use_state(cx, || initial_state);
    let cancel_id = use_state(cx, || None::<i32>);
    let frame_id = use_state(cx, || 0_i32);

    use_effect(cx, (frame_running,), |(frame_running,)| {
        to_owned![cancel_id, frame_id, frame_running];

        // frame_loop_holder holds a closure that is passed to request_animation_frame().
        // This closure is called each time an animation frame completes.
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

/// The code of use_animation_frame() is different enough for the desktop version that I made it a separate
/// function, though the interface is identical.
///
/// The desktop version does not have access to the web_sys crate, which gives access to the web context
/// of the browser's DOM.  This is because in dioxus desktop, code is compiled natively, not to WASM, so the
/// web_sys bindings do not exist.
///
/// Unitl Dioxus builds out additional libraries to access the DOM from desktop, we must use Dioxus' use_eval()
/// which executes given JavaScript inside WebKit, which has access to the DOM, and then comunicate with that
/// JavaScript code via send/recv.
///
/// The two pieces of code we need to set up are one to call window.requestAnimationFrame() recursively and update
/// the frame_id each time it is called, and another to call window.cancelAnimationFrame() to stop the above.
#[cfg(feature = "desktop")]
pub fn use_animation_frame(cx: Scope, initial_state: bool) -> (&UseState<bool>, &UseState<i32>) {
    let frame_running = use_state(cx, || initial_state);
    let cancel_id = use_state(cx, || None::<i32>);
    let frame_id = use_state(cx, || 0_i32);

    // Use eval returns a function that can spawn eval instances
    let create_eval = use_eval(cx);
    let run_eval = use_state(cx, || None::<UseEval>);

    // get_new_cancel_id is a future that waits for a new cancel_id from the javascript side (the run_eval command).
    let get_new_cancel_id = use_future(cx, (cancel_id, run_eval), |(_, run_eval)| {
        to_owned![run_eval];
        async move {
            // You can receive any message from JavaScript with the recv method
            if let Some(run_eval) = run_eval.get() {
                run_eval.recv().await.unwrap()
            } else {
                0.into()
            }
        }
    });

    // If we have a new cancel_id, save it to cancel_id and increment the frame_id.
    match get_new_cancel_id.value() {
        Some(remote_cancel_id) => {
            if let Ok(new_cancel_id) = remote_cancel_id.to_string().trim().parse() {
                if Some(new_cancel_id) != *cancel_id.get() {
                    cancel_id.with_mut(|cancel_id| {
                        *cancel_id = Some(new_cancel_id);
                    });

                    frame_id.with_mut(|id| {
                        *id = id.wrapping_add(1);
                    })
                }
            } else {
                println!("Could not convert javascript cancel_id value to number: {}", remote_cancel_id);
            }
        }
        _ => println!("Failed to get cancel_id from javascript"),
    };

    // TODO: use use_on_create to run create_eval() so that we only compile that code once.
    // use_on_create(cx, future)

    // If the value of frame_running has changed, either start frames running or stop them respectively.
    use_effect(cx, (frame_running,), |(frame_running,)| {
        to_owned![cancel_id, create_eval, run_eval];

        async move {
            // If we are requested to run, and we are not running, run
            if *frame_running.get() && run_eval.is_none() {
                run_eval.with_mut(|run_eval| {
                    *run_eval = Some(
                        create_eval(r#"
                            function gotFrame(last_render_ms) {
                                dioxus.send(window.requestAnimationFrame(gotFrame));
                            }
                            gotFrame(0);
                        "#)
                        .unwrap(),
                    );
                });
            }

            // If we are requested to stop, and we are running, stop
            if !*frame_running.get() {
                if let Some(cancel_id) = *cancel_id.get() {
                    let cancel_str = format!("window.cancelAnimationFrame({cancel_id});");

                    create_eval(&cancel_str).unwrap();
                    run_eval.set(None);
                }
            }
        }
    });

    (frame_running, frame_id)   
}