//! Entrypoint.
//! 
//! Adapted from the rust wasm tutorial: https://rustwasm.github.io/docs/book/game-of-life/introduction.html

#[macro_use]
pub(crate) mod websys_utils;
mod animation;
mod frames_per_second;
mod game_of_life;

use dioxus::prelude::*;

use crate::{frames_per_second::FramesPerSecond, animation::use_animation_frame, game_of_life::{GameOfLife, Redraw}, game_of_life::universe::Universe};

// use game_of_life::animation::use_animation_frame;
// use game_of_life::frames_per_second::FramesPerSecond;
// use game_of_life::game_of_life::{GameOfLife, Redraw};
// use game_of_life::game_of_life::universe::Universe;

fn main() {
    dioxus_web::launch(App);
}

#[component]
fn App(cx: Scope) -> Element {
    let (frames_running, frame_id) = use_animation_frame(cx, false);

    use_shared_state_provider(cx, Universe::new); // State of all cells in the universe
    use_shared_state_provider(cx, || Redraw::False); // Whether the universe needs to be redrawn

    let universe = use_shared_state::<Universe>(cx).unwrap();
    let redraw = use_shared_state::<Redraw>(cx).unwrap();

    render! {
        h2 { display: "flex", justify_content: "center", font_family: "Helvetica", "Game of Life" }
        GameOfLife { frame_id: *frame_id.get() }
        div { display: "flex", justify_content: "center",
            button { onclick: move |_| { frames_running.set(true) }, "Start" }
            button { onclick: move |_| { frames_running.set(false) }, "Stop" }
            button {
                onclick: move |_| {
                    universe
                        .with_mut(|universe| {
                            universe.tick();
                        });
                    redraw.with_mut(|redraw| { *redraw = Redraw::True })
                },
                "Step"
            }
        }
        div { display: "flex", justify_content: "center", FramesPerSecond { frame_id: *frame_id.get() } }
    }
}