//! Implementation of the GameOfLifeGrid component and supporting structures and methods.
//!
//! Adapted from the rust wasm tutorial: https://rustwasm.github.io/docs/book/game-of-life/introduction.html

use dioxus::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::{
    game_of_life::universe::{Cell, Universe, GRID_COLUMNS, GRID_ROWS},
    websys_utils::{into_2d_context, into_canvas_element},
};

mod universe;

const CELL_SIZE: u32 = 6; // px
const GRID_WIDTH: u32 = (CELL_SIZE + 1) * GRID_COLUMNS + 1;
const GRID_HEIGHT: u32 = (CELL_SIZE + 1) * GRID_ROWS + 1;

const GRID_COLOR: &str = "#CCCCCC";
const ALIVE_COLOR: &str = "#000000";
const DEAD_COLOR: &str = "#FFFFFF";

/// Draws the game of life grid, cells and buttons that can modify the universe.
///
/// frame_id represents each frame.  Each time the frame_id changes, the universe is advanced.
#[component]
pub fn GameOfLifeGrid(cx: Scope<'a>, frame_id: i32) -> Element {
    // State of all the cells in the universe.
    let universe = use_ref(cx, Universe::new);
    // Set by the "onmounted" event to give drawing functions access to the canvas element.
    let canvas_element = use_state(cx, || None::<web_sys::HtmlCanvasElement>);
    // Set true to redraw the cells.  Start as false as there is no need to draw an empty grid.
    let redraw = use_state(cx, || false);

    // Draw the grid when the canvas_element is created. Should happen only once.
    use_effect(cx, (canvas_element,), |(_,)| {
        to_owned![canvas_element];
        async move {
            if let Some(canvas_ele) = canvas_element.get() {
                draw_grid(canvas_ele);
            }
        }
    });

    // Advance and redraw the universe when the frame_id is changed.
    use_effect(cx, (frame_id,), |(_,)| {
        to_owned![universe, redraw];
        async move {
            universe.with_mut(|universe| {
                universe.tick();
                redraw.set(true);
            })
        }
    });

    // Redraw the universe when redraw is set to true (and set redraw to false).
    use_effect(cx, (redraw,), |(redraw,)| {
        to_owned![universe, canvas_element];
        async move {
            if *redraw.get() {
                if let Some(canvas_ele) = canvas_element.get() {
                    draw_cells(canvas_ele, universe.read().cells());
                }
                redraw.set(false);
            }
        }
    });

    render! {
        div { display: "flex", justify_content: "center",
            canvas {
                width: GRID_WIDTH as i64,
                height: GRID_HEIGHT as i64,
                onmounted: move |create_event| { canvas_element.set(get_canvas_element(create_event)) },
                onclick: move |mouse_event| { click_grid(mouse_event, universe, canvas_element) }
            }
        }
        div { display: "flex", justify_content: "center",
            button { onclick: move |_| { randomize_and_redraw(universe, redraw) }, "Random" }
            button { onclick: move |_| { clear_and_redraw(universe, redraw) }, "Clear" }
        }
    }
}

/// Randomize the universe and set the redraw signal.
fn randomize_and_redraw(universe: &UseRef<Universe>, redraw: &UseState<bool>) {
    universe.with_mut(|universe| {
        universe.random();
        redraw.set(true);
    });
}

/// Clear the universe and set the redraw signal.
fn clear_and_redraw(universe: &UseRef<Universe>, redraw: &UseState<bool>) {
    universe.with_mut(|universe| {
        universe.clear();
        redraw.set(true);
    });
}

/// Dig out the canvas element from the "onmount" event (which we get when the canvas element is created).
fn get_canvas_element(mount_event: dioxus::prelude::Event<dioxus::events::MountedData>) -> Option<HtmlCanvasElement> {
    if let Ok(Some(element)) = mount_event
        .get_raw_element()
        .map(|any| any.downcast_ref::<web_sys::Element>())
    {
        Some(into_canvas_element(element))
    } else {
        console_log!("mount_event should return a HtmlCanvasElement but did not");

        None
    }
}

/// Determine where the click was on the grid and toggle the appropriate cell.
fn click_grid(
    event: Event<MouseData>,
    universe: &UseRef<Universe>,
    canvas_element: &UseState<Option<web_sys::HtmlCanvasElement>>,
) {
    if let Some(canvas_ele) = canvas_element.get() {
        let coords = event.inner().client_coordinates();
        let bounding_rect = canvas_ele.get_bounding_client_rect();
        let width = canvas_ele.width() as f64;
        let scale_x = width / bounding_rect.width();
        let scale_y = width / bounding_rect.width();
        let canvas_left = coords.x - bounding_rect.left() * scale_x;
        let canvas_top = coords.y - bounding_rect.top() * scale_y;

        let row = (canvas_top / (CELL_SIZE + 1) as f64)
            .floor()
            .min((GRID_HEIGHT - 1) as f64) as u32;
        let col = (canvas_left / (CELL_SIZE + 1) as f64)
            .floor()
            .min((GRID_HEIGHT - 1) as f64) as u32;

        universe.with_mut(|universe| {
            universe.toggle_cell(row, col);
            draw_cells(canvas_ele, universe.cells());
        });
    } else {
        console_log!("Clicked on canvas element before it exists")
    }
}

// Draw the grid lines which contain the game of life cells.
//
// Only drawn once at startup as the cells are inbetween the grid
// and thus don't effect the grid pixels.
fn draw_grid(canvas_ele: &HtmlCanvasElement) {
    let context = into_2d_context(canvas_ele);

    let grid_color = JsValue::from_str(GRID_COLOR);
    let height = canvas_ele.height();
    let width = canvas_ele.width();

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

// Draw all the given cells in the grid.
//
// Cells should be a reference to all the cells in the universe.
fn draw_cells(canvas_ele: &HtmlCanvasElement, cells: &[Cell]) {
    let context = into_2d_context(canvas_ele);

    context.begin_path();

    fill_cells(&context, cells, Cell::Alive);
    fill_cells(&context, cells, Cell::Dead);
}

// Fill all the cells of the grid of the given cell_type (dead or alive).
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
