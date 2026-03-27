use std::{
    fs::{OpenOptions, create_dir_all},
    path::PathBuf,
};

use anyhow::{Result, anyhow};
use image::{GrayImage, Luma};
use log::{debug, info, trace};

use crate::{
    cell::{Cell, CellRegion, CellSpace, Generation, Region, State},
    structure::{GrayScale, Space},
    torus::{
        GrayScaleTorus, Tiling, Torus,
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

    fn update_all(&mut self, generation: &Gen) -> Result<()> {
        for cell in &self.cells {
            trace!("Update: [{:?}]", cell.id());
            let space = CellSpace;
            cell.update(&space, generation)?;
        }
        Ok(())
    }
}

impl<S, Gen> Space<S, Gen> for CellTorus<S, Gen>
where
    S: State<Gen>,
    Gen: Generation,
{
    type Reg = CellRegion<Self, S, Gen>;
    type Loc = Cell<S, Gen>;

    fn regions(&self, _generation: &Gen) -> impl IntoIterator<Item = Self::Reg> {
        let region = CellRegion::default();
        [region]
    }
}

impl<Spc, S, Gen> GrayScaleTorus<Spc, S, Gen> for CellTorus<S, Gen>
where
    Spc: Space<S, Gen, Loc = Cell<S, Gen>>,
    S: State<Gen> + GrayScale,
    Gen: Generation,
{
    fn export(
        &self,
        _region: &Spc::Reg,
        generation: &Gen,
        context: &<S as GrayScale>::Context,
        export_dir: Option<&PathBuf>,
    ) -> Result<()> {
        if let Some(dir) = export_dir {
            create_dir_all(&dir)?;
            match self.tiling {
                Tiling::Hexagons => export::<S, Gen>(self, generation, context, &dir)?,
                _ => todo!(),
            }
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
    let region: CellRegion<CellSpace, S, Gen> = CellRegion::default();
    let mut line = prefix.to_string();
    for x in 0..width {
        let s = (region.state(&cells[(x + offset) % width], generation) as Option<S>)
            .map(|s| format!("{s}"))
            .unwrap_or("?".to_string());
        line.push_str(&s);
        line.push_str(sep);
    }
    line
}

fn export<S, Gen>(
    torus: &CellTorus<S, Gen>,
    generation: &Gen,
    context: &<S as GrayScale>::Context,
    dir: &PathBuf,
) -> Result<()>
where
    S: State<Gen> + GrayScale,
    Gen: Generation,
{
    if torus.dimensions.len() != 2 {
        return Err(anyhow!("Torus should be two-dimensional"));
    }
    let height = torus.dimensions[0];
    let width = torus.dimensions[1];
    let mut img = GrayImage::new((width * 4 + 2) as u32, (height * 3 + 1) as u32);

    let region: CellRegion<CellTorus<S, Gen>, S, Gen> = CellRegion::default();
    let mut offset = 0;
    for y in 0..height {
        let line = &torus.cells[offset..(offset + width)];
        let xs = if (y % 2) == 0 { 2 } else { 0 };
        for x in 0..width {
            let gray = (region.state(&line[x], generation) as Option<S>)
                .map(|s| s.gray_value(context))
                .unwrap_or(128);
            let luma = [gray];
            let xo = (xs + 4 * x) as u32;
            let yo = 3 * y as u32;
            for xp in [1, 2] {
                for yp in 0..=3 {
                    img.put_pixel(xo + xp, yo + yp, Luma::from(luma.clone()));
                }
            }
            for xp in [0, 3] {
                for yp in [1, 2] {
                    img.put_pixel(xo + xp, yo + yp, Luma::from(luma.clone()));
                }
            }
        }
        offset = offset + width;
    }

    let mut file_path = dir.clone();
    file_path.push(format!("gen-{generation:?}.png"));
    let mut writer = OpenOptions::new()
        .create(true)
        .write(true)
        .open(file_path)?;
    img.write_to(&mut writer, image::ImageFormat::Png)?;
    Ok(())
}
