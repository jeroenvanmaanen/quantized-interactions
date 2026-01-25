use anyhow::{Result, anyhow};
use log::{debug, info, trace};

use crate::cell::{Cell, State};

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum Tiling {
    Orthogonal,
    OrthogonalAndDiagonal,
    AdjacentTriangles,
    TouchingTriangles,
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
            Tiling::OrthogonalAndDiagonal => connect_orthogonally_and_diagonally(&torus)?,
            Tiling::Hexagons => connect_hexagons(&torus)?,
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
            Tiling::OrthogonalAndDiagonal => {
                orthogonal_to_strings(&self.cells, &self.dimensions, generation, &mut lines)
            }
            Tiling::Hexagons => {
                hexagons_to_strings(&self.cells, &self.dimensions, generation, &mut lines);
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
fn connect_orthogonally_and_diagonally<S: State>(torus: &Torus<S>) -> Result<()> {
    connect_orthogonally(torus)?;
    connect_diagonally(torus)?;
    Ok(())
}

fn connect_orthogonally<S: State>(torus: &Torus<S>) -> Result<()> {
    let cells = &torus.cells;
    let mut co_ordinates = Vec::<usize>::new();
    let dimensionality = torus.dimensions.len();
    for _ in 0..dimensionality {
        co_ordinates.push(0);
    }
    for i in 0..cells.len() {
        assert!(get_index(&co_ordinates, &torus.dimensions)? == i);
        let center = &cells[i];
        for k in 0..dimensionality {
            let mut other = co_ordinates.clone();
            for d in &[torus.dimensions[k] - 1, 1] {
                other[k] = (co_ordinates[k] + d) % torus.dimensions[k];
                let other_index = get_index(&other, &torus.dimensions)?;
                trace!(
                    "Join neighbors: ({:?}) <=> ({:?}) ~ {} <=> {}",
                    &co_ordinates, &other, i, other_index
                );
                let neighbor = &cells[other_index];
                center.join(neighbor)?;
            }
        }
        next_co_ordinates(&mut co_ordinates, &torus.dimensions);
    }
    Ok(())
}

fn connect_diagonally<S: State>(torus: &Torus<S>) -> Result<()> {
    let cells = &torus.cells;
    let mut co_ordinates = Vec::<usize>::new();
    let dimensionality = torus.dimensions.len();
    for _ in 0..dimensionality {
        co_ordinates.push(0);
    }
    for i in 0..cells.len() {
        assert!(get_index(&co_ordinates, &torus.dimensions)? == i);
        let center = &cells[i];
        let corner_ids: usize = 1 << dimensionality;
        for c in 0..corner_ids {
            let mut corner = Vec::new();
            let mut bits = c;
            for k in 0..dimensionality {
                let offset = if bits & 1 == 1 {
                    1
                } else {
                    torus.dimensions[k] - 1
                };
                bits >>= 1;
                corner.push((co_ordinates[k] + offset) % torus.dimensions[k])
            }
            let corner_index = get_index(&corner, &torus.dimensions)?;
            trace!(
                "Join corner co-ordinates: ({:?}) <=> ({:?}) ~ {} <=> {}",
                &co_ordinates, &corner, i, corner_index
            );
            center.join(&cells[corner_index])?;
        }
        next_co_ordinates(&mut co_ordinates, &torus.dimensions);
    }
    Ok(())
}

fn next_co_ordinates(co_ordinates: &mut [usize], dimensions: &[usize]) {
    let dimensionality = dimensions.len();
    let mut j = dimensionality - 1;
    loop {
        co_ordinates[j] += 1;
        if co_ordinates[j] < dimensions[j] {
            break;
        } else {
            co_ordinates[j] = 0;
            if j < 1 {
                break;
            }
            j -= 1;
        }
    }
}

pub fn get_index(co_ordinates: &[usize], dimensions: &[usize]) -> Result<usize> {
    let dimensionality = dimensions.len();
    if co_ordinates.len() != dimensionality {
        return Err(anyhow!(
            "Sizes differ: {} != {}",
            co_ordinates.len(),
            dimensions.len()
        ));
    }
    let mut result = co_ordinates[0];
    if dimensionality > 1 {
        for offset in 0..dimensionality - 1 {
            result = result * dimensions[offset] + co_ordinates[offset + 1];
        }
    }
    Ok(result)
}

fn orthogonal_to_strings<S: State>(
    cells: &[Cell<S>],
    dimensions: &[usize],
    generation: &S::Gen,
    result: &mut Vec<String>,
) {
    let dimensionality = dimensions.len();
    if dimensionality > 1 {
        result.push("".to_string());
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
        result.push(line_to_string(cells, dimensions[0], generation, 0, "", ""));
    }
}

fn connect_hexagons<S: State>(torus: &Torus<S>) -> Result<()> {
    if torus.dimensions.len() != 2 {
        return Err(anyhow!("Tiling with triangles is only possible in 2-D"));
    }
    let height = torus.dimensions[0];
    let width = torus.dimensions[1];
    if (height % 2) == 1 || (width % 2) == 1 {
        return Err(anyhow!(
            "Tiling with triangles is only possible if both dimensions are even"
        ));
    }
    let cells = &torus.cells;
    let mut co_ordinates = vec![0, 0];
    for i in 0..cells.len() {
        assert!(get_index(&co_ordinates, &torus.dimensions)? == i);
        let center = &cells[i];
        let y = (co_ordinates[0] + 1) % height;
        let offset = width - (co_ordinates[0]) % 2;
        let lx = (co_ordinates[1] + offset) % width;
        let rx = (co_ordinates[1] + offset + 1) % width;
        let ax = (co_ordinates[1] + 1) % width;
        let left_index = get_index(&[y, lx], &torus.dimensions)?;
        let right_index = get_index(&[y, rx], &torus.dimensions)?;
        let next_index = get_index(&[co_ordinates[0], ax], &torus.dimensions)?;
        debug!(
            "Join left hexagon: ({:?}) <=> ({:?}) ~ {} <=> {}",
            &co_ordinates,
            &[lx, y],
            i,
            left_index
        );
        debug!(
            "Join right hexagon: ({:?}) <=> ({:?}) ~ {} <=> {}",
            &co_ordinates,
            &[rx, y],
            i,
            right_index
        );
        debug!(
            "Join next hexagon: ({:?}) <=> ({:?}) ~ {} <=> {}",
            &co_ordinates,
            &[ax, co_ordinates[1]],
            i,
            next_index
        );
        let left = &cells[left_index];
        let right = &cells[right_index];
        let next = &cells[next_index];
        center.join(&left)?;
        center.join(&right)?;
        center.join(&next)?;
        next_co_ordinates(&mut co_ordinates, &torus.dimensions);
    }
    Ok(())
}

fn hexagons_to_strings<S: State>(
    cells: &[Cell<S>],
    dimensions: &[usize],
    generation: &S::Gen,
    result: &mut Vec<String>,
) {
    let height = dimensions[0];
    let width = dimensions[1];
    let mut start = 0;
    for y in 0..height {
        let prefix = if (y % 2) == 0 { " " } else { "" };
        result.push(line_to_string(
            &cells[start..start + width],
            width,
            generation,
            0,
            prefix,
            " ",
        ));
        start += width;
    }
}

fn line_to_string<S: State>(
    cells: &[Cell<S>],
    width: usize,
    generation: &S::Gen,
    offset: usize,
    prefix: &str,
    sep: &str,
) -> String {
    let mut line = prefix.to_string();
    for x in 0..width {
        let s = cells[(x + offset) % width]
            .state(generation)
            .map(|s| format!("{s}"))
            .unwrap_or("?".to_string());
        line.push_str(&s);
        line.push_str(sep);
    }
    line
}
