use anyhow::{Result, anyhow};
use log::{debug, info, trace};

use crate::cell::{Cell, State};

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum Tiling {
    AdjacentTriangles,
    TouchingTriangles,
    AdjacentSquares,
    TouchingSquares,
    Hexagons,
}

pub struct Torus<S: State> {
    tiling: Tiling,
    width: usize,
    height: usize,
    cells: Vec<Cell<S>>,
}

impl<S: State> Torus<S> {
    pub fn new<C, F>(
        origin: C,
        tiling: Tiling,
        width: usize,
        height: usize,
        initial_gen: S::Gen,
        initial_state: F,
    ) -> Result<Torus<S>>
    where
        C: Into<Cell<S>>,
        F: Fn(&[usize]) -> S,
    {
        let dimensions = [width, height];
        let mut cardinality = 1usize;
        for i in 0..dimensions.len() {
            cardinality *= dimensions[i];
        }
        let mut cells = Vec::with_capacity(cardinality);
        let mut co_ordinates = Vec::new();
        create_cells(
            &mut co_ordinates,
            &dimensions,
            &mut cells,
            &initial_gen,
            &initial_state,
        );
        cells.push(origin.into());
        cells.swap_remove(0);
        debug!("Torus: Number of cells: [{}]", cells.len());

        let torus = Torus {
            tiling,
            width,
            height,
            cells,
        };

        match tiling {
            Tiling::TouchingSquares => connect_touching_squares(&torus)?,
            Tiling::AdjacentSquares => connect_adjacent_squares(&torus)?,
            _ => todo!(),
        }

        Ok(torus)
    }

    pub fn info(&self, generation: &S::Gen) {
        info!("Generation: {generation:?}");
        let lines = match self.tiling {
            Tiling::AdjacentSquares => squares_to_strings(self, generation),
            Tiling::TouchingSquares => squares_to_strings(self, generation),
            _ => todo!(),
        };
        for line in lines {
            info!("Line: [{line}]")
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&Cell<S>> {
        let i = (x % self.width) + (y % self.height) * self.width;
        self.cells.get(i)
    }

    pub fn update_all(&self, generation: &S::Gen) -> Result<()> {
        for i in 0..(self.width * self.height) {
            trace!("Update: [{i:?}]");
            if let Some(cell) = self.cells.get(i) {
                cell.update(generation)?;
            }
        }
        Ok(())
    }
}

fn create_cells<S: State, F>(
    co_ordinates: &mut Vec<usize>,
    dimensions: &[usize],
    cells: &mut Vec<Cell<S>>,
    initial_gen: &S::Gen,
    initial_state: &F,
) where
    F: Fn(&[usize]) -> S,
{
    co_ordinates.push(0);
    if dimensions.len() == 1 {
        for i in 0..dimensions[0] {
            co_ordinates.pop();
            co_ordinates.push(i);
            let state = initial_state(co_ordinates);
            cells.push(Cell::new(initial_gen.clone(), state));
        }
    } else if dimensions.len() > 1 {
        let subdimensions = &dimensions[1..];
        for i in 0..dimensions[0] {
            co_ordinates.pop();
            co_ordinates.push(i);
            create_cells(
                co_ordinates,
                subdimensions,
                cells,
                initial_gen,
                initial_state,
            );
        }
    }
    co_ordinates.pop();
}

fn connect_touching_squares<S: State>(torus: &Torus<S>) -> Result<()> {
    let table = &torus.cells;
    let width = torus.width;
    let height = torus.height;
    if width <= 0 || height <= 0 {
        return Err(anyhow!("Torus too small: <{width}, {height}>"));
    }
    for x in 0..width {
        for y in 0..height {
            if let Some(center) = table.get(x + y * width) {
                let lx = x + width - 1;
                for bx in 0..3 {
                    let xx = (lx + bx) % width;
                    let ty = y + height - 1;
                    for by in 0..3 {
                        let yy = (ty + by) % width;
                        if xx != x || yy != y {
                            if let Some(neighbor) = table.get(xx + yy * width) {
                                center.join(neighbor)?;
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn connect_adjacent_squares<S: State>(torus: &Torus<S>) -> Result<()> {
    let table = &torus.cells;
    let width = torus.width;
    let height = torus.height;
    if width <= 0 || height <= 0 {
        return Err(anyhow!("Torus too small: <{width}, {height}>"));
    }
    for x in 0..width {
        for y in 0..height {
            if let Some(center) = table.get(x + y * width) {
                let lx = x + width - 1;
                for (bx, by) in &[(width - 1, 0), (0, height - 1), (1, 0), (0, 1)] {
                    let xx = (lx + bx) % width;
                    let ty = y + height - 1;
                    let yy = (ty + by) % width;
                    if let Some(neighbor) = table.get(xx + yy * width) {
                        center.join(neighbor)?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn squares_to_strings<S: State>(torus: &Torus<S>, generation: &S::Gen) -> Vec<String> {
    let mut result = Vec::with_capacity(torus.height);
    for y in 0..torus.height {
        let mut line = "".to_string();
        for x in 0..torus.width {
            let char = torus
                .get(x, y)
                .and_then(|c| c.state(generation))
                .map(|s| s.to_char())
                .unwrap_or('?');
            line.push(char);
        }
        result.push(line);
    }
    result
}
