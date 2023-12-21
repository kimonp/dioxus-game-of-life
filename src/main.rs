#![allow(non_snake_case)]

use dioxus::html::GlobalAttributes;
// use dioxus_elements::canvas;
// import the prelude to get access to the `rsx!` macro and the `Scope` and `Element` types
use dioxus::prelude::*;

use game_of_life::bindgen_glue::{cancel_animation_frame, document, request_animation_frame};
use game_of_life::console_log;
use game_of_life::frames_per_second::FramesPerSecond;
use game_of_life::universe::{Cell, Universe, GRID_COLUMNS, GRID_ROWS};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::CanvasRenderingContext2d;

use std::cell::RefCell;
use std::rc::Rc;

const CANVAS_ID: &str = "game-of-life-canvas";
const CELL_SIZE: u32 = 6; // px
const GRID_WIDTH: u32 = (CELL_SIZE + 1) * GRID_COLUMNS + 1;
const GRID_HEIGHT: u32 = (CELL_SIZE + 1) * GRID_ROWS + 1;

const GRID_COLOR: &str = "#CCCCCC";
const ALIVE_COLOR: &str = "#FFFFFF";
const DEAD_COLOR: &str = "#000000";

extern crate console_error_panic_hook;
use std::panic;

use wasm_bindgen::prelude::wasm_bindgen;
#[wasm_bindgen]
pub fn init_panic_hook() {
    // Better logging of panics in the browser
    console_error_panic_hook::set_once();
}

fn main() {
    // launch the dioxus app in a webview
    // dioxus_desktop::launch(App);
    console_log!("Starting...");
    dioxus_web::launch(App);
}

// define a component that renders a div with the text "Hello, world!"
fn App(cx: Scope) -> Element {
    let test_string = String::from("test");

    render! {
        Universe { name: test_string }
        FramesPerSecond {}
    }
}

fn get_2d_context() -> CanvasRenderingContext2d {
    let canvas_ele = document().get_element_by_id(CANVAS_ID).unwrap();
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

fn config_canvas(universe: Rc<Mutex<Universe>>) {
    if let Some(canvas_ele) = document().get_element_by_id(CANVAS_ID) {
        console_log!("Configuring canvas...");

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

                universe.lock().unwrap().toggle_cell(row, col);

                draw_cells(&universe.lock().unwrap());

                // console_log!("Row: {row} Col: {col}");
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

fn draw_grid() {
    let context = get_2d_context();
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

fn draw_cells(universe: &Universe) {
    let context = get_2d_context();
    let cells = universe.cells();
    let alive_color = JsValue::from_str(ALIVE_COLOR);
    let dead_color = JsValue::from_str(DEAD_COLOR);

    context.begin_path();

    context.set_fill_style(&alive_color);
    fill_cells(&context, cells, Cell::Alive);

    context.set_fill_style(&dead_color);
    fill_cells(&context, cells, Cell::Dead);
}

fn fill_cells(context: &CanvasRenderingContext2d, cells: &[Cell], cell_type: Cell) {
    for row in 0..GRID_ROWS {
        for col in 0..GRID_COLUMNS {
            let index = get_grid_index(row, col);
            let cell = cells.get(index as usize);

            if let Some(cell) = cell {
                if cell == &cell_type {
                    continue;
                }
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

use std::sync::Mutex;
pub fn update_frame_loop(universe: Rc<Mutex<Universe>>) {
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut frames_per_second = FramesPerSecond::new("frames-per-second");
    let mut check = false;

    // Does not need to be FnMut if universe is behing a mutex
    *g.borrow_mut() = Some(Closure::<dyn FnMut()>::new(move || {
        frames_per_second.update_frame();
        let id = request_animation_frame(f.borrow().as_ref().unwrap());

        if !check {
            config_canvas(universe.clone());
            check = true;
        }
        draw_grid();
        draw_cells(&universe.lock().unwrap());
        universe.lock().unwrap().tick();
        document()
            .get_element_by_id("animation-id")
            .unwrap()
            .set_text_content(Some(&id.to_string()))
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());
}

#[component]
fn Universe(cx: Scope, name: String) -> Element {
    let universe = Rc::new(Mutex::new(Universe::new()));
    update_frame_loop(universe.clone());
    config_canvas(universe.clone());

    let mut count = use_state(cx, || 0);
    let height = universe.lock().unwrap().height();
    let width = universe.lock().unwrap().width();
    let universe_clear = universe.clone();
    let universe_random = universe.clone();

    render! {
        div { display: "flex", justify_content: "center",
            button {
                onclick: move |_| {
                    let animation_id_element = document().get_element_by_id("animation-id").unwrap();
                    let animation_id_text = animation_id_element.text_content();
                    if let Some(animation_id_text) = animation_id_text {
                        if let Ok(animation_id) = animation_id_text.parse::<i32>() {
                            cancel_animation_frame(animation_id);
                            animation_id_element.set_text_content(None);
                        }
                    }
                },
                "Stop"
            }
            button {
                onclick: move |_| {
                    let animation_id_element = document().get_element_by_id("animation-id").unwrap();
                    let animation_id_text = animation_id_element.text_content();
                    if let Some(animation_id_text) = animation_id_text {
                        if let Ok(animation_id) = animation_id_text.parse::<i32>() {
                            cancel_animation_frame(animation_id);
                            animation_id_element.set_text_content(None);
                        }
                        update_frame_loop(universe.clone());
                    }
                },
                "Start"
            }
            button {
                onclick: move |_| {
                    universe_clear.lock().unwrap().clear();
                    draw_cells(&universe_clear.lock().unwrap());
                },
                "Clear"
            }
            button {
                onclick: move |_| {
                    universe_random.lock().unwrap().random();
                    draw_cells(&universe_random.lock().unwrap());
                },
                "Random"
            }
        }
        div { display: "flex", justify_content: "center", canvas { id: CANVAS_ID } }
        div {
            // button { onclick: move |_| { universe.lock().unwrap().set_height(height + 1) }, "Universe Height Up" }
            // button {
            //     onclick: move |_| {
            //         universe.lock().unwrap().set_height(height - 1);
            //     },
            //     "Universe Height Down"
            // }
        }
        div { hidden: false, color: "green", font_family: "arial", padding: "0.5rem", position: "relative",
            "Universe Size: {height}, {width}"
        }
        button { onclick: move |_| { count += 1 }, "Up" }
        button { onclick: move |_| { count -= 1 }, "Down" }
        div { hidden: false, color: "green", padding: "0.5rem", position: "relative", font_family: "verdana",
            "Counter: {count} {name.to_uppercase()}"
        }
    }
}

fn FramesPerSecond(cx: Scope) -> Element {
    render! {
        div { id: "frames-per-second", white_space: "pre", font_family: "monospace" }
        div { id: "animation-id", white_space: "pre", font_family: "monospace" }
    }
}
