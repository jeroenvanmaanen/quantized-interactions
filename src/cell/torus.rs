use std::borrow::Cow;

use anyhow::{Result, anyhow};
use log::{debug, info, trace};

use crate::{
    cell::{Cell, CellRegion, CellSpace, Generation, Region, State},
    structure::Space,
    torus::{
        Tiling, Torus,
        utils::{get_index, next_co_ordinates},
    },
};

pub struct CellTorus<S: State<Gen>, Gen: Generation> {
    tiling: Tiling,
    dimensions: Vec<usize>,
    cells: Vec<Cell<S, Gen>>,
}

pub fn new_cell_torus<S: State<Gen>, Gen: Generation, F>(
    tiling: Tiling,
    dimensions: &[usize],
    initial_gen: Gen,
    initial_state: F,
) -> Result<CellTorus<S, Gen>>
where
    F: Fn(&[usize]) -> S,
{
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
        0,
    );
    debug!("Torus: Number of cells: [{}]", cells.len());

    let torus = CellTorus {
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

impl<S: State<Gen>, Gen: Generation> Torus<S, Gen> for CellTorus<S, Gen> {
    type Spc = Self;

    fn space(&self) -> &Self::Spc {
        self
    }

    fn space_mut(&mut self) -> &mut Self::Spc {
        self
    }

    fn info(&self, generation: &Gen) {
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

    fn update_all_cells(&mut self, generation: &Gen) -> Result<()> {
        self.update_all(generation)
    }

    fn tiling(&self) -> Tiling {
        self.tiling
    }

    fn dimensions(&self) -> Vec<usize> {
        self.dimensions.clone()
    }

    fn adjust(&mut self, generation: &Gen, x: usize, y: usize, state: S) -> Result<()> {
        let index = y * self.dimensions[1] + x;
        let mut write_lock = self.cells[index]
            .0
            .state_map
            .write()
            .map_err(|e| anyhow!("Could not get write lock: {e}"))?;
        write_lock.get_mut(generation).map(|s| *s = state);
        Ok(())
    }

    fn coordinates(
        &self,
        _region: &<Self::Spc as Space<S, Gen>>::Reg,
        location: &<Self::Spc as Space<S, Gen>>::Loc,
    ) -> (usize, usize) {
        let width = self.dimensions[0];
        let index = location.0.index;
        (index % width, index / width)
    }
}

impl<S, Gen> Space<S, Gen> for CellTorus<S, Gen>
where
    S: State<Gen>,
    Gen: Generation,
{
    type Reg = CellRegion<Self, S, Gen>;
    type Loc = Cell<S, Gen>;

    fn regions(&self, generation: &Gen) -> impl IntoIterator<Item = Self::Reg> {
        let region = CellRegion::new(generation.clone());
        [region]
    }

    fn region<'a>(
        &'a self,
        generation: &Gen,
        _location: &Self::Loc,
    ) -> Option<std::borrow::Cow<'a, Self::Reg>> {
        Some(Cow::Owned(CellRegion::new(generation.clone())))
    }

    fn update_all(&mut self, generation: &Gen) -> Result<()> {
        for cell in &self.cells {
            trace!("Update: [{:?}]", cell.id());
            let space = CellSpace;
            cell.update(&space, generation)?;
        }
        Ok(())
    }

    fn locations(&self, _region: &Self::Reg) -> impl IntoIterator<Item = Self::Loc> {
        self.cells.clone()
    }

    fn free(&mut self, generation: &Gen) -> Result<()> {
        for cell in &self.cells {
            cell.0
                .state_map
                .write()
                .map_err(|e| anyhow!("Can't get write lock on cells: {e:?}"))?
                .remove(generation);
        }
        Ok(())
    }
}

fn create_cells<S: State<Gen>, Gen: Generation, F>(
    co_ordinates: &mut Vec<usize>,
    dimensions: &[usize],
    cells: &mut Vec<Cell<S, Gen>>,
    initial_gen: &Gen,
    initial_state: &F,
    start_index: usize,
) -> usize
where
    F: Fn(&[usize]) -> S,
{
    let next_index: usize;
    co_ordinates.push(0);
    if dimensions.len() == 1 {
        for i in 0..dimensions[0] {
            co_ordinates.pop();
            co_ordinates.push(i);
            let state = initial_state(co_ordinates);
            cells.push(Cell::new_with_index(
                initial_gen.clone(),
                state,
                start_index + i,
            ));
        }
        next_index = start_index + dimensions[0];
    } else if dimensions.len() > 1 {
        let mut start_index = start_index;
        let subdimensions = &dimensions[1..];
        for i in 0..dimensions[0] {
            co_ordinates.pop();
            co_ordinates.push(i);
            start_index = create_cells(
                co_ordinates,
                subdimensions,
                cells,
                initial_gen,
                initial_state,
                start_index,
            );
        }
        next_index = start_index;
    } else {
        next_index = start_index;
    }
    co_ordinates.pop();
    next_index
}

fn connect_orthogonally_and_diagonally<S, Gen>(torus: &CellTorus<S, Gen>) -> Result<()>
where
    S: State<Gen>,
    Gen: Generation,
{
    connect_orthogonally(torus)?;
    connect_diagonally(torus)?;
    Ok(())
}

fn connect_orthogonally<S: State<Gen>, Gen: Generation>(torus: &CellTorus<S, Gen>) -> Result<()> {
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
                    "Join effectors: ({:?}) <=> ({:?}) ~ {} <=> {}",
                    &co_ordinates, &other, i, other_index
                );
                let effector = &cells[other_index];
                center.join(effector)?;
            }
        }
        next_co_ordinates(&mut co_ordinates, &torus.dimensions);
    }
    Ok(())
}

fn connect_diagonally<S: State<Gen>, Gen: Generation>(torus: &CellTorus<S, Gen>) -> Result<()> {
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

fn orthogonal_to_strings<S: State<Gen>, Gen: Generation>(
    cells: &[Cell<S, Gen>],
    dimensions: &[usize],
    generation: &Gen,
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

fn connect_hexagons<S: State<Gen>, Gen: Generation>(torus: &CellTorus<S, Gen>) -> Result<()> {
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

fn hexagons_to_strings<S: State<Gen>, Gen: Generation>(
    cells: &[Cell<S, Gen>],
    dimensions: &[usize],
    generation: &Gen,
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

fn line_to_string<S: State<Gen>, Gen: Generation>(
    cells: &[Cell<S, Gen>],
    width: usize,
    generation: &Gen,
    offset: usize,
    prefix: &str,
    sep: &str,
) -> String {
    let region: CellRegion<CellSpace, S, Gen> = CellRegion::new(generation.clone());
    let mut line = prefix.to_string();
    for x in 0..width {
        let s = (region.state(&cells[(x + offset) % width]) as Option<S>)
            .map(|s| format!("{s}"))
            .unwrap_or("?".to_string());
        line.push_str(&s);
        line.push_str(sep);
    }
    line
}
