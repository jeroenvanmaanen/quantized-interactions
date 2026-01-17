use anyhow::{Result, anyhow};
use log::{debug, info, trace};

use crate::cell::{Cell, Generation, State};

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
        F: Fn(usize, usize) -> S,
    {
        let mut cells = Vec::with_capacity(width * height);
        for x in 0..width {
            for y in 0..height {
                let state = initial_state(x, y);
                cells.push(Cell::new(initial_gen.clone(), state));
            }
        }
        cells.push(origin.into());
        cells.swap_remove(0);
        debug!("Torus: Number of cells: [{}]", cells.len());

        match tiling {
            Tiling::TouchingSquares => connect_touching_squares(&cells, width, height)?,
            _ => todo!(),
        }
        let torus = Torus {
            tiling,
            width,
            height,
            cells,
        };
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

fn connect_touching_squares<G, S>(table: &Vec<Cell<S>>, width: usize, height: usize) -> Result<()>
where
    G: Generation,
    S: State<Gen = G>,
{
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
