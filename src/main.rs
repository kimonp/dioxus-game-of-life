//! Entrypoint.
//! 
//! Adapted from the rust wasm tutorial: https://rustwasm.github.io/docs/book/game-of-life/introduction.html

use dioxus::prelude::*;

use game_of_life::animation::use_animation_frame;
use game_of_life::frames_per_second::FramesPerSecond;
use game_of_life::game_of_life::GameOfLifeGrid;

fn main() {
    dioxus_web::launch(App);
}

#[component]
fn App(cx: Scope) -> Element {
    let (frames_running, frame_id) = use_animation_frame(cx, false);

    render! {
        h2 { display: "flex", justify_content: "center", font_family: "Helvetica", "Game of Life" }
        GameOfLifeGrid { frame_id: *frame_id.get() }
        div { display: "flex", justify_content: "center",
            button { onclick: move |_| { frames_running.set(true) }, "Start" }
            button { onclick: move |_| { frames_running.set(false) }, "Stop" }
        }
        FramesPerSecond { frame_id: *frame_id.get() }
    }
}