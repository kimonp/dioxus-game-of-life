//! Implementation of the GameOfLifeGrid component and supporting structures and methods.

use dioxus::prelude::*;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::CanvasRenderingContext2d;

use crate::game_of_life::universe::{Cell, Universe, GRID_COLUMNS, GRID_ROWS};
use crate::websys_utils::*;

mod universe;

const CANVAS_ID: &str = "game-of-life-canvas";
const CELL_SIZE: u32 = 6; // px
const GRID_WIDTH: u32 = (CELL_SIZE + 1) * GRID_COLUMNS + 1;
const GRID_HEIGHT: u32 = (CELL_SIZE + 1) * GRID_ROWS + 1;

const GRID_COLOR: &str = "#CCCCCC";
const ALIVE_COLOR: &str = "#000000";
const DEAD_COLOR: &str = "#FFFFFF";

/// Draws the game of life grid, cells and buttons that can modify the universe.
#[component]
pub fn GameOfLifeGrid(cx: Scope<'a>, frame_id: i32) -> Element {
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

// Configure the grid.
//
// * Set the height and width, based on the universe.
// * Set an onclick event_listener to toggle cells when clicked.
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

// Draw all the given cells in the grid.
//
// Cells should be a reference to all the cells in the universe.
fn draw_cells(cells: &[Cell]) {
    let context = get_2d_context(CANVAS_ID);
    // let cells = universe.cells();

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
