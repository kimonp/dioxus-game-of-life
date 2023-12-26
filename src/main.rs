// use dioxus_elements::canvas;
// import the prelude to get access to the `rsx!` macro and the `Scope` and `Element` types
use dioxus::prelude::*;

use game_of_life::animation::use_animation_frame;
use game_of_life::frames_per_second::FramesPerSecond;
use game_of_life::game_of_life::GameOfLifeGrid;

// Entry point
fn main() {
    // launch the dioxus app in a webview
    // dioxus_desktop::launch(App);
    dioxus_web::launch(App);
}

#[component]
fn App(cx: Scope) -> Element {
    let (frames_running, frame_id) = use_animation_frame(cx, false);

    render! {
        GameOfLifeGrid { frame_id: *frame_id.get() }
        div { display: "flex", justify_content: "center",
            button {
                onclick: move |_| {
                    frames_running.set(true);
                },
                "Start"
            }
            button {
                onclick: move |_| {
                    frames_running.set(false);
                },
                "Stop"
            }
        }
        FramesPerSecond { frame_id: *frame_id.get() }
    }
}

// use web_sys::HtmlElement;

// #[component]
// fn Focus(cx: Scope) -> Element {
//     let test = use_ref(cx, || None::<i32>);
//     let input_element = use_ref(cx, || None::<HtmlElement>);
  
//     // input { r#type: "text", r#ref: input_element }
//     // input { r#type: "text" },
//     // button { onclick: focus_input, "Focus Input" }
//     render! {
//         input { r#type: "text", ty: move |_| { input_element } }
//         button {
//             onclick: move |_| {
//                 input_element
//                     .with(|input_element| {
//                         input_element
//                             .clone()
//                             .map(|input_element| {
//                                 let _ = input_element.focus();
//                                 input_element
//                             });
//                     });
//             },
//             "Focus Input"
//         }
//     }
// }