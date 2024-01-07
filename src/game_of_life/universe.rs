//! Implements the game of life universe, which is represented by a grid of cells.
#[cfg(feature = "desktop")]
use rand::Rng;

#[cfg(feature = "web")]
use web_sys::js_sys::Math;

pub const CELLS_PER_ROW: u32 = 64;
pub const CELLS_PER_COL: u32 = CELLS_PER_ROW;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

impl Cell {
    fn toggle(&mut self) {
        *self = match *self {
            Cell::Dead => Cell::Alive,
            Cell::Alive => Cell::Dead,
        };
    }
}

/// Represents the state of all cells in the universe.
#[derive(Eq, PartialEq)]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
}

impl Default for Universe { fn default() -> Self { Self::new() } }

impl Universe {
    /// Create a new universe with the standard height and width.
    pub fn new() -> Universe {
        let width = CELLS_PER_ROW;
        let height = CELLS_PER_COL;

        let cells = (0..width * height).map(|_i| Cell::Dead).collect();

        Universe {
            width,
            height,
            cells,
        }
    }

    // Randomly set the value of all cells in the universe.
    //
    // 6 out of 10 cells on average are set to be alive.
    pub fn random(&mut self) { 
        #[cfg(feature = "desktop")]
        let mut rng = rand::thread_rng();

        self.cells = (0..self.width * self.height)
            .map(|_i| {
                #[cfg(feature = "desktop")]
                let random = rng.gen_range(0..10);
                #[cfg(feature = "web")]
                let random = get_random_int(10);

                if random > 3 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();
    }

    // Return a reference to all cells.
    #[allow(unused)]
    pub fn cells(&self) -> &Vec<Cell> {
        &self.cells
    }

    // Return a Vector of tuples of the (x,y) coordinates of all cells that are currently alive.
    pub fn get_living_cells(&self) -> Vec<(i64, i64)> {
        let mut cells = Vec::new();

        for col in 0..self.width {
            for row in 0..self.height {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];

                if cell == Cell::Alive {
                    cells.push((col as i64, row as i64));
                }
            }
        }
        cells
    }

    /// Advance the universe one tick.
    ///
    /// Kill dead cells and spawn new ones depending the neigbor count of each cell.
    pub fn tick(&mut self) {
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                let next_cell = match (cell, live_neighbors) {
                    // Rule 1: Any live cell with fewer than two live neighbours
                    // dies, as if caused by underpopulation.
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    // Rule 2: Any live cell with two or three live neighbours
                    // lives on to the next generation.
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    // Rule 3: Any live cell with more than three live
                    // neighbours dies, as if by overpopulation.
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    // Rule 4: Any dead cell with exactly three live neighbours
                    // becomes a live cell, as if by reproduction.
                    (Cell::Dead, 3) => Cell::Alive,
                    // All other cells remain in the same state.
                    (otherwise, _) => otherwise,
                };

                next[idx] = next_cell;
            }
        }

        self.cells = next;
    }

    // Clear all cells in the universe.
    pub fn clear(&mut self) {
        self.cells = (0..self.width * self.height).map(|_i| Cell::Dead).collect();
    }

    /// Toggle the state of the cell at row, column.
    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.cells[idx].toggle();
    }

    /// Return the index of the cell at row, column.
    fn get_index(&self, row: u32, col: u32) -> usize {
        (row * self.width + col) as usize
    }

    /// Return the count of live cells around cell at row, column.
    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                count += self.cells[idx] as u8;
            }
        }
        count
    }
}

#[cfg(feature = "web")]
fn get_random_int(max: u32) -> u32 {
    Math::abs(Math::floor(Math::random() * max as f64)) as u32
}