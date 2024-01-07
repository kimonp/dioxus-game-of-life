//! Entry point for the Game of Life.
//!
//! Adapted from the rust wasm tutorial: https://rustwasm.github.io/docs/book/game-of-life/introduction.html

#[cfg(feature = "web")]
#[macro_use]
pub(crate) mod websys_utils;

pub(crate) mod animation;
pub(crate) mod frames_per_second;
pub(crate) mod game_of_life;

use dioxus::{html::GlobalAttributes, prelude::*};

use crate::{
    animation::use_animation_frame,
    frames_per_second::FramesPerSecond,
    game_of_life::universe::Universe,
    game_of_life::{GameOfLife, Redraw},
};

fn main() {
    #[cfg(feature = "web")]
    dioxus_web::launch(App);

    #[cfg(feature = "desktop")]
    launch_desktop();
}

/// Size and position the application window and launch the desktop app.
#[cfg(feature = "desktop")]
fn launch_desktop() {
    use dioxus_desktop::{tao::dpi::LogicalPosition, Config, PhysicalSize, WindowBuilder};
    use game_of_life::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};

    // TODO: Now that the grid is an SVG, scale the grid when the window changes
    let size = PhysicalSize::new(
        GRID_WIDTH * 2.0 + (CELL_SIZE * 2) as f64 * 2.0,
        GRID_HEIGHT * 2.0 + 400.0,
    );
    let position = LogicalPosition::new(10, 10);
    let window = WindowBuilder::new()
        .with_title("Game of Life")
        .with_inner_size(size)
        .with_position(position);

    dioxus_desktop::launch_with_props(App, (), Config::new().with_window(window));
}

/// Top component in the DOM.
#[component]
fn App(cx: Scope) -> Element {
    let (frames_running, frame_id) = use_animation_frame(cx, false);

    use_shared_state_provider(cx, Universe::new); // State of all cells in the universe
    use_shared_state_provider(cx, || Redraw::False); // True if the universe needs to be redrawn

    render! {
        h2 { display: "flex", justify_content: "center", font_family: "Helvetica", "Game of Life" }
        div { display: "grid", justify_content: "center", GameOfLife { frame_id: *frame_id.get() } }
        div { display: "flex", justify_content: "center",
            button { onclick: move |_| { frames_running.set(true) }, "Start" }
            button { onclick: move |_| { frames_running.set(false) }, "Stop" }
            StepButton {}
        }
        div { display: "flex", justify_content: "center", FramesPerSecond { frame_id: *frame_id.get() } }
    }
}

// Advance the universe one step when clicked.
#[component]
fn StepButton(cx: Scope) -> Element {
    let universe = use_shared_state::<Universe>(cx).unwrap();
    let redraw = use_shared_state::<Redraw>(cx).unwrap();

    render! {
        button { onclick: move |_| {
                universe
                    .with_mut(|universe| {
                        universe.tick();
                    });
                redraw.with_mut(|redraw| { *redraw = Redraw::True })
            },
            "Step"
        }
    }
}
