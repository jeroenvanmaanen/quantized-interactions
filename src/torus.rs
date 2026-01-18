use anyhow::{Result, anyhow};
use log::{debug, info, trace};

use crate::cell::{Cell, State};

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum Tiling {
    Orthogonal,
    AdjacentTriangles,
    TouchingTriangles,
    TouchingSquares,
    Hexagons,
}

pub struct Torus<S: State> {
    tiling: Tiling,
    dimensions: Vec<usize>,
    cells: Vec<Cell<S>>,
}

impl<S: State> Torus<S> {
    pub fn new<C, F>(
        origin: C,
        tiling: Tiling,
        dimensions: &[usize],
        initial_gen: S::Gen,
        initial_state: F,
    ) -> Result<Torus<S>>
    where
        C: Into<Cell<S>>,
        F: Fn(&[usize]) -> S,
    {
        let mut cardinality = 1usize;
        for i in 0..dimensions.len() {
            cardinality *= dimensions[i];
        }
        let mut cells = Vec::with_capacity(cardinality);
        if cardinality > 1 {
            let mut co_ordinates = Vec::new();
            create_cells(
                &mut co_ordinates,
                &dimensions,
                &mut cells,
                &initial_gen,
                &initial_state,
            );
            cells[0] = origin.into();
        } else {
            cells.push(origin.into());
        }
        debug!("Torus: Number of cells: [{}]", cells.len());

        let torus = Torus {
            tiling,
            dimensions: dimensions.into(),
            cells,
        };

        match tiling {
            Tiling::Orthogonal => connect_orthogonally(&torus)?,
            Tiling::TouchingSquares => connect_touching_squares(&torus)?,
            _ => todo!(),
        }

        Ok(torus)
    }

    pub fn info(&self, generation: &S::Gen) {
        info!("Generation: {generation:?}");
        let mut lines = Vec::new();
        match self.tiling {
            Tiling::Orthogonal => {
                orthogonal_to_strings(&self.cells, &self.dimensions, generation, &mut lines)
            }
            Tiling::TouchingSquares => {
                orthogonal_to_strings(&self.cells, &self.dimensions, generation, &mut lines)
            }
            _ => todo!(),
        };
        for line in lines {
            info!("Line: [{line}]")
        }
    }

    pub fn update_all(&self, generation: &S::Gen) -> Result<()> {
        for cell in &self.cells {
            trace!("Update: [{:?}]", cell.id());
            cell.update(generation)?;
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

fn connect_orthogonally<S: State>(torus: &Torus<S>) -> Result<()> {
    let cells = &torus.cells;
    let mut co_ordinates = Vec::<usize>::new();
    let dimensionality = torus.dimensions.len();
    for _ in 0..dimensionality {
        co_ordinates.push(0);
    }
    for i in 0..cells.len() {
        let mut j = dimensionality - 1;
        loop {
            co_ordinates[j] += 1;
            if co_ordinates[j] < torus.dimensions[j] {
                break;
            } else {
                co_ordinates[j] = 0;
                if j < 1 {
                    break;
                }
                j -= 1;
            }
        }
        let center = &cells[i];
        for k in 0..dimensionality {
            for d in &[torus.dimensions[k] - 1, 1] {
                let mut other = co_ordinates.clone();
                other[k] = (other[k] + d) % torus.dimensions[k];
                let neighbor = &cells[get_index(&co_ordinates, &torus.dimensions)?];
                center.join(neighbor)?;
            }
        }
    }
    Ok(())
}

fn get_index(co_ordinates: &[usize], dimensions: &[usize]) -> Result<usize> {
    let dimensionality = dimensions.len();
    if co_ordinates.len() != dimensionality {
        return Err(anyhow!(
            "Sizes differ: {} != {}",
            co_ordinates.len(),
            dimensions.len()
        ));
    }
    let mut result = co_ordinates[dimensionality - 1];
    if dimensionality > 1 {
        for k in 0..dimensionality - 1 {
            let offset = dimensionality - k - 1;
            result = result * dimensions[offset] + co_ordinates[offset - 1];
        }
    }
    Ok(result)
}

fn connect_touching_squares<S: State>(torus: &Torus<S>) -> Result<()> {
    let table = &torus.cells;
    let width = torus.dimensions[0];
    let height = torus.dimensions[1];
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

fn orthogonal_to_strings<S: State>(
    cells: &[Cell<S>],
    dimensions: &[usize],
    generation: &S::Gen,
    result: &mut Vec<String>,
) {
    let dimensionality = dimensions.len();
    if dimensionality > 2 {
        result.push("".to_string());
    }
    if dimensionality > 1 {
        let mut width = 1;
        for k in 1..dimensionality {
            width = width * dimensions[k];
        }
        for i in 0..dimensions[0] {
            let start = i * width;
            orthogonal_to_strings(
                &cells[start..(start + width)],
                &dimensions[1..],
                generation,
                result,
            );
        }
    } else if dimensionality == 1 {
        result.push(line_to_string(cells, dimensions[0], generation));
    }
}

fn line_to_string<S: State>(cells: &[Cell<S>], width: usize, generation: &S::Gen) -> String {
    let mut line = "".to_string();
    for x in 0..width {
        let char = cells[x]
            .state(generation)
            .map(|s| s.to_char())
            .unwrap_or('?');
        line.push(char);
    }
    line
}
