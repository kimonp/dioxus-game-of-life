use std::cell::RefCell;
use std::rc::Rc;

use dioxus::html::GlobalAttributes;
// use dioxus_elements::canvas;
// import the prelude to get access to the `rsx!` macro and the `Scope` and `Element` types
use dioxus::prelude::*;

use game_of_life::frames_per_second::FramesPerSecond;
use game_of_life::universe::{Cell, Universe, GRID_COLUMNS, GRID_ROWS};
use game_of_life::websys_utils::*;
use game_of_life::{console_log, frames_per_second};

use wasm_bindgen::prelude::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::CanvasRenderingContext2d;

const CANVAS_ID: &str = "game-of-life-canvas";
const ANIMATION_ELEMENT_ID: &str = "animation-id-element";
const ANIMATION_ATTRIBUTE: &str = "animation-id";
const CELL_SIZE: u32 = 6; // px
const GRID_WIDTH: u32 = (CELL_SIZE + 1) * GRID_COLUMNS + 1;
const GRID_HEIGHT: u32 = (CELL_SIZE + 1) * GRID_ROWS + 1;

const GRID_COLOR: &str = "#CCCCCC";
const ALIVE_COLOR: &str = "#000000";
const DEAD_COLOR: &str = "#FFFFFF";

// extern crate console_error_panic_hook;
// use std::panic;

// #[wasm_bindgen]
// pub fn init_panic_hook() {
//     // Better logging of panics in the browser
//     console_error_panic_hook::set_once();
// }

// Entry point
fn main() {
    // launch the dioxus app in a webview
    // dioxus_desktop::launch(App);
    dioxus_web::launch(App);
}

// A custom Dioxus hook that abstracts the request_animation_frame() and cancel_animation_frame() DOM calls.
//
// Allows the caller to create a use_effect which watches the frame_id which can then
// take an action each time a frame is advanced.
//
// Returns two UseState variables: frame_running and frame_id.
// * frame_running is true if frames are advancing.
// * frame_id is incremented each time a new frame is run.
//
// If frame_running is set to true, frames advance.
// If frame_running is set to false, frames stop advancing.
fn use_animation_frame(cx: Scope, initial_state: bool) -> (&UseState<bool>, &UseState<i32>) {
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

#[component]
fn GameOfLifeGrid(cx: Scope<'a>, frame_id: i32) -> Element {
    let universe = use_ref(cx, Universe::new);

    use_on_create(cx, || {
        to_owned![universe];
        async move {
            config_grid(universe);
        }
    });

    use_effect(cx, (frame_id,), |(_frame_id,)| {
        to_owned![universe];
        async move {
            universe.with_mut(|universe| {
                universe.tick();
                draw_cells(universe.cells());
            })
        }
    });

    render! {
        div { display: "flex", justify_content: "center", canvas { id: CANVAS_ID } }
        div { display: "flex", justify_content: "center",
            button {
                onclick: move |_| {
                    universe
                        .with_mut(|universe| {
                            universe.random();
                        });
                    draw_cells(universe.read().cells());
                },
                "Random"
            }
            button {
                onclick: move |_| {
                    universe
                        .with_mut(|universe| {
                            universe.clear();
                        });
                    draw_cells(universe.read().cells());
                },
                "Clear"
            }
        }
    }
}

fn config_grid(universe: UseRef<Universe>) {
    if let Some(canvas_ele) = document().get_element_by_id(CANVAS_ID) {
        let canvas_ele: web_sys::HtmlCanvasElement = canvas_ele
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();

        if canvas_ele.height() == GRID_HEIGHT {
            console_log!("Canvas already configured...");
            return;
        }
        canvas_ele.set_height(GRID_HEIGHT);
        canvas_ele.set_width(GRID_WIDTH);

        draw_grid();

        let universe = universe.clone();
        let toggle_cell_closure =
            Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
                let canvas_ele = document().get_element_by_id(CANVAS_ID).unwrap();
                let canvas_ele: web_sys::HtmlCanvasElement = canvas_ele
                    .dyn_into::<web_sys::HtmlCanvasElement>()
                    .map_err(|_| ())
                    .unwrap();
                let bounding_rect = canvas_ele.get_bounding_client_rect();

                let scale_x = canvas_ele.width() as f64 / bounding_rect.width();
                let scale_y = canvas_ele.width() as f64 / bounding_rect.width();

                let canvas_left = event.client_x() as f64 - bounding_rect.left() * scale_x;
                let canvas_top = event.client_y() as f64 - bounding_rect.top() * scale_y;

                let row = (canvas_top / (CELL_SIZE + 1) as f64)
                    .floor()
                    .min((GRID_HEIGHT - 1) as f64) as u32;
                let col = (canvas_left / (CELL_SIZE + 1) as f64)
                    .floor()
                    .min((GRID_HEIGHT - 1) as f64) as u32;

                universe.with_mut(|universe| {
                    universe.toggle_cell(row, col);
                    draw_cells(universe.cells());
                });
            });
        let _ = canvas_ele.add_event_listener_with_callback(
            "click",
            toggle_cell_closure.as_ref().unchecked_ref(),
        );
        toggle_cell_closure.forget();
    } else {
        console_log!("Could not find id: {CANVAS_ID}");
    }
}


// Frames per second component that shows how quickly the app is rendering animation frames.
#[component]
fn FramesPerSecond(cx: Scope, frame_id: i32) -> Element {
    let frames_per_second = use_ref(cx, FramesPerSecond::new);
    let fps_text = use_state(cx, || frames_per_second.read().text());

    // console_log!("Running app: {:?}", frame_id.get());

    use_effect(cx, (frame_id,), |(_frame_id,)| {
        to_owned![frames_per_second, fps_text];
        async move {
            frames_per_second.with_mut(|fps| {
                fps.update_frame();
                fps_text.modify(|_old_text| fps.text());
            });
        }
    });

    render! {
        div { white_space: "pre", font_family: "monospace", fps_text.get().clone() }
    }
}

// Draw the grid lines which contain the game of life cells.
fn draw_grid() {
    let context = get_2d_context(CANVAS_ID);
    let grid_color = JsValue::from_str(GRID_COLOR);
    let height = GRID_HEIGHT;
    let width = GRID_WIDTH;

    context.begin_path();
    context.set_line_width(0.5);
    context.set_stroke_style(&grid_color);

    // Vertical lines
    for i in 0..=width {
        let x = (i * (CELL_SIZE + 1) + 1) as f64;
        let y = ((CELL_SIZE + 1) * height + 1) as f64;
        context.move_to(x, 0_f64);
        context.line_to(x, y)
    }

    // Horizontal lines
    for i in 0..=height {
        let x = ((CELL_SIZE + 1) * width + 1) as f64;
        let y = (i * (CELL_SIZE + 1) + 1) as f64;
        context.move_to(0_f64, y);
        context.line_to(x, y)
    }

    context.stroke();
}

fn get_grid_index(row: u32, col: u32) -> u32 {
    row * GRID_ROWS + col
}

// Draw all cells in the grid based on the state of the universe.
fn draw_cells(cells: &Vec<Cell>) {
    let context = get_2d_context(CANVAS_ID);
    // let cells = universe.cells();

    context.begin_path();

    fill_cells(&context, cells, Cell::Alive);
    fill_cells(&context, cells, Cell::Dead);
}

// Fill all the cells of the grid of the given cell_type (dead or alive).
//
// Use javascript canvas API for the rendering.
fn fill_cells(context: &CanvasRenderingContext2d, cells: &[Cell], cell_type: Cell) {
    let fill_color = JsValue::from_str(match cell_type {
        Cell::Alive => ALIVE_COLOR,
        Cell::Dead => DEAD_COLOR,
    });
    context.set_fill_style(&fill_color);

    for row in 0..GRID_ROWS {
        for col in 0..GRID_COLUMNS {
            let index = get_grid_index(row, col);

            if let Some(cell) = cells.get(index as usize) {
                if cell == &cell_type {
                    context.fill_rect(
                        (col * (CELL_SIZE + 1) + 1) as f64,
                        (row * (CELL_SIZE + 1) + 1) as f64,
                        CELL_SIZE as f64,
                        CELL_SIZE as f64,
                    )
                }
            }
        }
    }
}