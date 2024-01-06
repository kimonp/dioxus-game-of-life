//! Implementation of the GameOfLifeGrid component and supporting structures and methods.
//!
//! Adapted from the rust wasm tutorial: https://rustwasm.github.io/docs/book/game-of-life/introduction.html

pub mod universe;

use dioxus::prelude::*;
use universe::{Universe, CELLS_PER_COL, CELLS_PER_ROW};

pub const GRID_ROWS: i64 = CELLS_PER_ROW as i64;
pub const GRID_COLUMNS: i64 = CELLS_PER_COL as i64;

const SMALL_GRID_STROKE: f64 = 0.5;
const BIG_GRID_STROKE: f64 = 1.0;
// Cells need to be smaller than the grid so they don't cover it.  Since the grid stroke
// straddles the virtual grid line, the adjustment is half the grid stroke size.
const SMALL_GRID_STROKE_OFFSET: f64 = SMALL_GRID_STROKE / 2.0;
const BIG_GRID_STROKE_OFFSET: f64 = BIG_GRID_STROKE / 2.0;

pub const CELL_SIZE: i64 = 8; // px

pub const CELLS_WIDTH: f64 = (CELL_SIZE * GRID_COLUMNS) as f64;
pub const CELLS_HEIGHT: f64 = (CELL_SIZE * GRID_ROWS) as f64;

pub const GRID_WIDTH: f64 = CELLS_WIDTH + BIG_GRID_STROKE;
pub const GRID_HEIGHT: f64 = CELLS_HEIGHT + BIG_GRID_STROKE;

const BIG_GRID_MULTIPLIER: i64 = 8;
const BIG_GRID_SIZE: i64 = CELL_SIZE * BIG_GRID_MULTIPLIER;

const SMALL_GRID_COLOR: &str = "#CCCCCC";
const GRID_COLOR: &str = "gray";
const ALIVE_COLOR: &str = "#000000";

#[derive(Eq, PartialEq)]
pub enum Redraw {
    True,
    False,
}

impl Redraw {
    fn is_true(&self) -> bool {
        *self == Redraw::True
    }
}

/// Draws the game of life grid, cells and buttons that can modify the universe.
///
/// frame_id represents each frame.  Each time the frame_id changes, the universe is advanced.
///
/// Split into two components: the grid, and the cells.
///
/// The grid is slightly bigger than the cells because of the stroke volume of the big grid.
/// Thus we must shift the cells by that amount to center them on the grid.
#[component]
pub fn GameOfLife(cx: Scope<'a>, frame_id: i32) -> Element {
    // State of all the cells in the universe.
    let universe = use_shared_state::<Universe>(cx).unwrap();
    // Set true to redraw the cells.  Start as false as there is no need to draw an empty grid.
    let redraw = use_shared_state::<Redraw>(cx).unwrap();
    // List of the coordiantes of all currently living cells in the universe.
    let living_cells = use_ref(cx, || universe.read().get_living_cells());

    let svg_element = use_state(cx, || None::<web_sys::Element>);

    // Advance and redraw the universe when the frame_id is changed.
    use_effect(cx, (frame_id,), |(_,)| {
        to_owned![universe, redraw];
        async move {
            universe.with_mut(|universe| {
                universe.tick();
            });
            redraw.with_mut(|redraw| {
                *redraw = Redraw::True;
            });
        }
    });

    // Redraw the universe when redraw is set to true (and set redraw to false).
    use_effect(cx, (redraw,), |(redraw,)| {
        to_owned![universe, living_cells];
        async move {
            if redraw.read().is_true() {
                living_cells.with_mut(|living_cells| {
                    *living_cells = universe.read().get_living_cells();
                });
                redraw.with_mut(|redraw| {
                    *redraw = Redraw::False;
                });
            }
        }
    });

    render! {
        svg {
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
            onmounted: move |create_event| { svg_element.set(get_element_from_event(create_event)) },
            g { transform: "translate({BIG_GRID_STROKE_OFFSET},{BIG_GRID_STROKE_OFFSET})",
                GameOfLifeCells { live_cells: living_cells.read().clone() }
            }
            GameOfLifeGrid {}
        }
        div { display: "flex", justify_content: "center",
            button { onclick: move |_| { randomize_and_redraw(universe, redraw) }, "Random" }
            button { onclick: move |_| { clear_and_redraw(universe, redraw) }, "Clear" }
        }
    }
}

/// Randomize the universe and set the redraw signal.
fn randomize_and_redraw(universe: &UseSharedState<Universe>, redraw: &UseSharedState<Redraw>) {
    universe.with_mut(|universe| {
        universe.random();
    });
    redraw.with_mut(|redraw| {
        *redraw = Redraw::True;
    });
}

/// Clear the universe and set the redraw signal.
fn clear_and_redraw(universe: &UseSharedState<Universe>, redraw: &UseSharedState<Redraw>) {
    universe.with_mut(|universe| {
        universe.clear();
    });
    redraw.with_mut(|redraw| {
        *redraw = Redraw::True;
    });
}

/// Dig out the canvas element from the "onmount" event (which we get when the canvas element is created).
fn get_element_from_event(mount_event: dioxus::prelude::Event<dioxus::events::MountedData>) -> Option<web_sys::Element> {
    if let Ok(Some(element)) = mount_event
        .get_raw_element()
        .map(|any| any.downcast_ref::<web_sys::Element>())
    {
        Some(element.clone())
    } else {
        console_log!("mount_event should return a HtmlCanvasElement but did not");

        None
    }
}

/// Determine where the click was on the grid and toggle the appropriate cell.
fn click_grid(
    event: Event<MouseData>,
    universe: &UseSharedState<Universe>,
    redraw: &UseSharedState<Redraw>,
) {
    // TODO: This should be the bounding rect of the grid element,a but I don't yet have an easy
    // way to get the grid element itself from the DOM.  When we need is the actual width and height
    // of the element.
    //
    // This works for now because it assumes the rectangle of the grid is not scaled.
    // This would not be true if we scaled the element based on the size of the window for example.
    let element_width = GRID_WIDTH;
    let element_height = GRID_HEIGHT;

    let coords = event.element_coordinates();
    let scale_x = GRID_WIDTH / element_width;
    let scale_y = GRID_HEIGHT / element_height;

    let scaled_x = coords.x * scale_x;
    let scaled_y = coords.y * scale_y;

    let col = (scaled_x / (CELL_SIZE as f64)).floor().min(GRID_HEIGHT) as u32;
    let row = (scaled_y / (CELL_SIZE as f64)).floor().min(GRID_WIDTH) as u32;

    // println!("coords: {:?}, col,row: {col},{row}", coords);
    universe.with_mut(|universe| {
        universe.toggle_cell(row, col);
    });
    redraw.with_mut(|redraw| {
        *redraw = Redraw::True;
    });
}

/// Draw the grid lines that hold the cells in the game of life.
///
/// Defines two SVG patterns:
///  * smallGrid, which is a square the size of a cell that draws thin grid lines
///  * grid, which is the a square the size of N cells that draws a thicker grid line
///
/// The rectangle that defines the grid is then filled with these two paterns.
///
/// Each of the patterns nees to be shifted by half the stroke width so that the
/// line is shown fully.
#[component]
pub fn GameOfLifeGrid(cx: Scope) -> Element {
    let universe = use_shared_state::<Universe>(cx).unwrap();
    let redraw = use_shared_state::<Redraw>(cx).unwrap();

    // Needed to center the small grid on the big grid
    let small_adj = BIG_GRID_STROKE_OFFSET - SMALL_GRID_STROKE_OFFSET;

    render! {
        svg { onclick: move |mouse_event| click_grid(mouse_event, universe, redraw),
            defs {
                pattern { id: "smallGrid", width: CELL_SIZE, height: CELL_SIZE, pattern_units: "userSpaceOnUse",
                    g { transform: "translate({SMALL_GRID_STROKE_OFFSET},{SMALL_GRID_STROKE_OFFSET})",
                        path {
                            d: "M {CELL_SIZE} 0 L 0 0 0 {CELL_SIZE}",
                            fill: "none",
                            stroke: SMALL_GRID_COLOR,
                            stroke_width: SMALL_GRID_STROKE
                        }
                    }
                }
                pattern { id: "grid", width: BIG_GRID_SIZE, height: BIG_GRID_SIZE, pattern_units: "userSpaceOnUse",
                    g { transform: "translate({small_adj},{small_adj})", rect { width: BIG_GRID_SIZE, height: BIG_GRID_SIZE, fill: "url(#smallGrid)" } }
                    g { transform: "translate({BIG_GRID_STROKE_OFFSET},{BIG_GRID_STROKE_OFFSET})",
                        path {
                            d: "M {BIG_GRID_SIZE} 0 L 0 0 0 {BIG_GRID_SIZE}",
                            fill: "none",
                            stroke: GRID_COLOR,
                            stroke_width: BIG_GRID_STROKE
                        }
                    }
                }
            }
            rect { width: "100%", height: "100%", fill: "url(#grid)" }
        }
    }
}

/// Render all the given live cells.
///
/// We set the viewbox, and thus the local coordiantes, to be based on the number of cells.
#[component]
pub fn GameOfLifeCells(cx: Scope, live_cells: Vec<(i64, i64)>) -> Element {
    let rendered_cells = live_cells
        .iter()
        .map(|(x, y)| rsx! { GameOfLifeCell { x: *x, y: *y } });

    render! {
        svg { view_box: "0 0 {CELLS_PER_ROW} {CELLS_PER_ROW}", width: CELLS_WIDTH, height: CELLS_HEIGHT, rendered_cells }
    }
}

/// Draw a single cell in the grid.
///
/// Note that when drawing a cell, the units of the viewport are the size of each cell.
/// Thus the x and y coordiates are the cell to be rendered, and the height and width are 1.
#[component]
pub fn GameOfLifeCell(cx: Scope, x: i64, y: i64) -> Element {
    render! { rect { x: *x, y: *y, width: 1, height: 1, fill: ALIVE_COLOR } }
}