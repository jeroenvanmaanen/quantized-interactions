mod info;

use anyhow::{Result, anyhow};
use log::{debug, warn};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    patch::{
        AtMostSixEffectors, Effectors, LocationInPatch, PatchLinks, SMALL_PATCH_SIZE,
        SmallIndexType, SmallPatch,
    },
    structure::{Generation, Space, State},
    torus::{Tiling, Torus},
};
use info::info_hexagons;

use super::Crystal;

pub struct PatchTorus<S: State<Gen> + Copy, Gen: Generation, PL: PatchLinks> {
    tiling: Tiling,
    dimensions: Vec<usize>,
    patch_grid: Vec<usize>,
    crystal: Crystal<S, Gen, PL>,
}

#[derive(Default)]
pub struct TorusPatchLinks {
    effectors: AtMostSixEffectors,
    edges: HashMap<SmallIndexType, (usize, SmallIndexType)>,
    total_width: SmallIndexType,
    total_height: SmallIndexType,
    inner_width: SmallIndexType,
    inner_height: SmallIndexType,
    even: bool,
}

impl PatchLinks for TorusPatchLinks {
    type Eff = AtMostSixEffectors;

    fn effectors(&self) -> &Self::Eff {
        &self.effectors
    }

    fn edges(&self) -> &std::collections::HashMap<SmallIndexType, (usize, SmallIndexType)> {
        &self.edges
    }
}

impl<S: State<Gen> + Copy, Gen: Generation> Torus<S, Gen> for PatchTorus<S, Gen, TorusPatchLinks> {
    type Spc = Crystal<S, Gen, TorusPatchLinks>;

    fn space(&self) -> &Self::Spc {
        &self.crystal
    }

    fn space_mut(&mut self) -> &mut Self::Spc {
        &mut self.crystal
    }

    fn info(&self, _generation: &Gen) {
        if self.tiling == Tiling::Hexagons {
            info_hexagons(self);
        } else {
            warn!("No info for tiling: [{:?}]", self.tiling)
        }
    }

    fn update_all_cells(&mut self, generation: &Gen) -> Result<()> {
        self.crystal.update_all(generation)
    }

    fn tiling(&self) -> Tiling {
        self.tiling
    }

    fn dimensions(&self) -> Vec<usize> {
        self.dimensions.clone()
    }

    fn adjust(&mut self, generation: &Gen, x: usize, y: usize, state: S) -> Result<()> {
        let mut p = 0;
        let mut px = x;
        while self.crystal.patch_links[p].inner_width as usize <= px {
            px -= self.crystal.patch_links[p].inner_width as usize;
            p += 1;
        }
        let (w, _) = calculate_grid(self.dimensions[0], self.dimensions[1]);
        let mut py = y;
        while self.crystal.patch_links[p].inner_height as usize <= py {
            py -= self.crystal.patch_links[p].inner_height as usize;
            p += w;
        }
        let patch_ref = &self.crystal.generations[generation][p];
        let mut patch = patch_ref.borrow_mut();
        let pi =
            py as SmallIndexType * self.crystal.patch_links[p].inner_width + px as SmallIndexType;
        patch.cells[pi as usize] = state;
        Ok(())
    }

    fn coordinates(
        &self,
        patch_ref: &Rc<RefCell<SmallPatch<S, Gen>>>,
        location: &LocationInPatch,
    ) -> (usize, usize) {
        let patch_links = &self.crystal.patch_links;
        let patch = patch_ref.borrow();
        let i = patch.index;
        let w = self.patch_grid[0];
        let r = i / w;
        let c = i % w;
        let mut left_x = 0;
        for p in 0..c {
            left_x += patch_links[p].inner_width as usize;
        }
        let mut top_y = 0;
        let mut top_i = 0;
        for _ in 0..r {
            top_y += patch_links[top_i].inner_height as usize;
            top_i += w;
        }
        let pw = patch_links[i].inner_width;
        let li = location.index;
        let ly = li / pw;
        let lx = li % pw;
        (left_x + lx as usize, top_y + ly as usize)
    }
}

const SQRT_PATCH_SIZE: u8 = 17;

pub fn new_hexagonal_torus<S: State<Gen> + Copy, Gen: Generation>(
    init: S,
    initial_gen: Gen,
    width: usize,
    height: usize,
) -> Result<PatchTorus<S, Gen, TorusPatchLinks>> {
    if width % 2 == 1 || height % 2 == 1 {
        return Err(anyhow!("Must both be even: ({width}, {height})"));
    }
    let dimensions = vec![width, height];
    let patch_links_factory = || TorusPatchLinks::default();
    let (w, h) = calculate_grid(width, height);
    let patch_grid = vec![w, h];
    let mut crystal = Crystal::new(w * h, &initial_gen, init, patch_links_factory);
    connect_cells_hexagonally(&mut crystal, width, w, height, h, &initial_gen)?;
    Ok(PatchTorus {
        crystal,
        dimensions,
        patch_grid,
        tiling: Tiling::Hexagons,
    })
}

fn calculate_grid(width: usize, height: usize) -> (usize, usize) {
    if width >= height {
        calculate_oblong(width, height)
    } else {
        let (v, h) = calculate_oblong(height, width);
        (h, v)
    }
}

fn calculate_oblong(long: usize, short: usize) -> (usize, usize) {
    let sps = SQRT_PATCH_SIZE as usize;
    let sm = (short + sps) / (sps + 1);
    let mut s = sm;
    let (mut l, mut q, mut e) = calculate_footprint(long, short, sm);
    if s > 1 {
        let (la, qa, ea) = calculate_footprint(long, short, sm - 1);
        if qa < q || (qa == q && ea < e) {
            s = sm - 1;
            l = la;
            q = qa;
            e = ea;
        }
    }
    let (lb, qb, eb) = calculate_footprint(long, short, sm + 1);
    if qb < q || (qb == q && eb < e) {
        s = sm + 1;
        l = lb;
    }
    debug!("Patch count: {l} x {s}");
    (l, s)
}

fn calculate_footprint(long: usize, short: usize, s: usize) -> (usize, usize, usize) {
    let sd = if s > 1 { 2 } else { 0 };
    let sx = sd + (short + s - 1) / s;
    let lx = (SMALL_PATCH_SIZE as usize) / sx;
    let ld = if lx < long { 2 } else { 0 };
    let l = (long + lx - ld - 1) / (lx - ld);
    let mut edge = 0;
    if s > 1 {
        edge += s * lx;
    }
    if l > 1 {
        edge += l * sx;
        if s > 1 {
            edge += 4;
        }
    }
    let footprint = (l, s * l, edge);
    let se = sx - sd;
    let le = lx - ld;
    debug!(
        "Footprint: ({long} / {l}, {short} / {s}) -> ({le} + {ld}, {se} + {sd}) -> {} / {}",
        footprint.1, footprint.2
    );
    footprint
}

fn connect_cells_hexagonally<S, Gen>(
    crystal: &mut Crystal<S, Gen, TorusPatchLinks>,
    width: usize,
    w: usize,
    height: usize,
    h: usize,
    generation: &Gen,
) -> Result<()>
where
    S: State<Gen> + Copy,
    Gen: Generation,
{
    debug!(
        "Connect cells: [{}]: ([{width}] / [{w}]) x ([{height}] / [{h}])",
        crystal.patch_count()
    );

    let even_offset_coords = vec![(0, -1), (1, -1), (-1, 0), (1, 0), (0, 1), (1, 1)];
    let odd_offset_coords = vec![(-1, -1), (0, -1), (-1, 0), (1, 0), (-1, 1), (0, 1)];
    let even_offsets = Alternatives::new(even_offset_coords, odd_offset_coords);

    let patch_grid = PatchGrid::new(width, w, height, h);
    let mut cell_rows_before = 0;
    let mut br = 0;
    for r in 0..h {
        let hr = patch_grid.row_height(r); // Height of this row
        let mut cell_colums_before = 0;
        for c in 0..w {
            let p = br + c;
            let wc = patch_grid.column_width(c); // Width of this column
            let wi = patch_grid.internal_column_width(c);
            let hi = patch_grid.internal_row_height(r);
            let internal_size = wi * hi;
            let even = cell_rows_before & 0x01 == 0; // TODO: is this correct?
            let patch_links = &mut crystal.patch_links[p];
            patch_links.total_width = wc;
            patch_links.total_height = hr;
            patch_links.inner_width = wi;
            patch_links.inner_height = hi;
            patch_links.even = even;
            debug!(
                "Patch: #{p}: [{r}]: [{c}]: ([{wc}] x [{hr}]): [{cell_colums_before}, {cell_rows_before}, {even}]"
            );
            if let Some(patches) = crystal.generations.get_mut(generation) {
                if let Some(patch_ref) = patches.get_mut(p) {
                    let mut patch = patch_ref.borrow_mut();
                    patch.size = internal_size;
                    patch.total_size = wc * hr;
                }
            }

            let shuffle = prepare_shuffle(wc, hr, w > 1, h > 1);

            let effectors = &mut patch_links.effectors;
            let mut offsets = even_offsets.clone();
            if !even {
                offsets = offsets.other();
            }
            let er = if h > 1 { 1 } else { 0 };
            let mut iy = er * wc;
            for y in er..(hr - er) {
                let ec = if w > 1 { 1 } else { 0 };
                for x in ec..(wc - ec) {
                    let i = iy + x;
                    for (ox, oy) in offsets.offsets() {
                        let xx = (x + ox + wc - 1) % wc;
                        let yy = (y + oy + hr - 1) % hr;
                        let j = (yy * wc) + xx;
                        effectors.add(shuffle(i), shuffle(j))?;
                    }
                }
                offsets = offsets.other();
                iy += wc;
            }
            effectors.debug(format!("{p}"));
            cell_colums_before += wc;

            let edges = &mut patch_links.edges;
            if h > 1 {
                let this_base = (hr - 1) * wc;
                let above = (r + h - 1) % h;
                let above_base = (patch_grid.internal_row_height(above) - 1) * wi;
                let below = (r + 1) % h;
                let fudge = if w > 1 { 1 } else { 0 };
                for i in fudge..(wc - fudge) {
                    edges.insert(shuffle(i), (above * w + c, above_base + i - fudge)); // top to bottom of above
                    edges.insert(shuffle(this_base + i), (below * w + c, i - fudge)); // bottom to top of below
                }
            }
            if w > 1 {
                let row = r * w;
                let this_base = wc - 1;
                let left = (c + w - 1) % w;
                let left_width = patch_grid.internal_column_width(left);
                let right = (c + 1) % w;
                let right_width = patch_grid.internal_column_width(right);
                let fudge = if h > 1 { 1 } else { 0 };
                let mut offset = fudge * wc;
                for i in fudge..(hr - fudge) {
                    edges.insert(
                        shuffle(offset),
                        (row + left, ((i + 1 - fudge) * left_width) - 1),
                    ); // leftmost cell to rightmost cell of patch to the left
                    edges.insert(
                        shuffle(this_base + offset),
                        (row + right, (i - fudge) * right_width),
                    ); // rightmost cell to leftmost cell of patch to the right
                    offset += wc;
                }
                if h > 1 {
                    // Insert corners
                    let above = (r + h - 1) % h;
                    let lft = (c + w - 1) % w;
                    let right = (c + 1) % w;
                    let below = (r + 1) % h;

                    let tlis = patch_grid.internal_size(above, left);
                    let tris = patch_grid.internal_size(above, right);
                    let lft_wi = patch_grid.internal_column_width(left);
                    let right_wi = patch_grid.internal_column_width(right);
                    edges.insert(shuffle(0), (above * w + lft, tlis - 1));
                    edges.insert(shuffle(wc - 1), (above * w + right, tris - right_wi));
                    edges.insert(shuffle((hr - 1) * wc), (below * w + lft, lft_wi - 1));
                    edges.insert(shuffle(hr * wc - 1), (below * w + right, 0));
                }
            }
        }
        cell_rows_before += hr;
        br += w;
    }
    Ok(())
}

/// If the grid is wide (more than one patch in the x-direction) as well as tall (more than one patch in the y-direction),
/// then a two by two patch has edges on both sides as well as corners. It is shuffled as follows:
///
/// Original:
/// ```
/// |  0 |  1 |  2 |  3 |
/// |  4 |  5 |  6 |  7 |
/// |  8 |  9 | 10 | 11 |
/// | 12 | 13 | 14 | 15 |
/// ```
///
/// Shuffled
/// ```
/// |  5 |  6 |            First row
/// |  9 | 10 |            Last row
/// |  0 |  1 |  2 |  3 |  Top edge
/// | 12 | 13 | 14 | 15 |  Bottom edge
/// |  4 |  8 |            Left edge, minus corners
/// |  7 | 11 |            Right edge, minus corners
/// ```
fn prepare_shuffle(
    wc: SmallIndexType,
    hr: SmallIndexType,
    wide: bool,
    tall: bool,
) -> impl Fn(SmallIndexType) -> SmallIndexType {
    let p_size = hr * wc;
    let mut shuffle = [0 as SmallIndexType; SMALL_PATCH_SIZE as usize];
    let mut y_start = 0;
    let mut y_end = p_size;
    let mut x_start = 0;
    let mut x_end = wc;
    if tall {
        y_start = wc;
        y_end = p_size - wc;
    }
    if wide {
        x_start = 1;
        x_end -= 1;
    }
    let mut i = 0;
    while y_start < y_end {
        for x in x_start..x_end {
            shuffle[(y_start + x) as usize] = i;
            i = i + 1;
        }
        y_start += wc;
    }
    if tall {
        for x in 0..wc {
            shuffle[x as usize] = i;
            i += 1;
        }
        for x in 0..wc {
            shuffle[(y_end + x) as usize] = i;
            i += 1;
        }
    }
    if wide {
        let fy = if tall { 1 } else { 0 };
        for y in (fy)..(hr - fy) {
            shuffle[(y * wc) as usize] = i;
            i += 1;
        }
        for y in (fy)..(hr - fy) {
            shuffle[(y * wc + wc - 1) as usize] = i;
            i += 1;
        }
    }
    assert!(i == p_size);
    move |i| shuffle[i as usize]
}

struct PatchGrid {
    wi: SmallIndexType, // Base internal width of patches
    wp: SmallIndexType, // Base width of patches
    wq: usize,          // Number of patch columns that are one cell wider
    hi: SmallIndexType, // Base internal height of patches
    hp: SmallIndexType, // Base height of patches
    hq: usize,          // Number of patch rows that are one cell wider
}

impl PatchGrid {
    fn new(width: usize, w: usize, height: usize, h: usize) -> Self {
        let wi = (width / w) as SmallIndexType; // With of a small patch
        let wq = width - w * (wi as usize); // Number of collums that are one cell wider
        let wp = if w > 1 { wi + 2 } else { wi };
        let wp = wp as SmallIndexType;
        let hi = (height / h) as SmallIndexType; // Height of a small patch
        let hq = height - h * (hi as usize); // Number of rows that are one cell taller
        let hp = if h > 1 { hi + 2 } else { hi };
        let hp = hp as SmallIndexType;
        PatchGrid {
            wi,
            wp,
            wq,
            hi,
            hp,
            hq,
        }
    }

    fn internal_row_height(&self, r: usize) -> SmallIndexType {
        self.hi + (if r < self.hq { 1 } else { 0 })
    }

    fn row_height(&self, r: usize) -> SmallIndexType {
        self.hp + (if r < self.hq { 1 } else { 0 })
    }

    fn internal_column_width(&self, c: usize) -> SmallIndexType {
        self.wi + (if c < self.wq { 1 } else { 0 })
    }

    fn column_width(&self, c: usize) -> SmallIndexType {
        self.wp + (if c < self.wq { 1 } else { 0 })
    }

    fn internal_size(&self, r: usize, c: usize) -> SmallIndexType {
        self.internal_row_height(r) * self.internal_column_width(c)
    }
}

struct Offsets {
    even: Vec<(SmallIndexType, SmallIndexType)>,
    odd: Vec<(SmallIndexType, SmallIndexType)>,
}

#[derive(Clone)]
enum Alternatives {
    Even(Rc<Offsets>),
    Odd(Rc<Offsets>),
}

impl Alternatives {
    fn new(even_coords: Vec<(i8, i8)>, odd_coords: Vec<(i8, i8)>) -> Self {
        let even = coords_to_small_index_type(even_coords);
        let odd = coords_to_small_index_type(odd_coords);
        Alternatives::Even(Rc::new(Offsets { even, odd }))
    }

    fn offsets(&self) -> &[(SmallIndexType, SmallIndexType)] {
        match self {
            Alternatives::Even(offsets) => &offsets.even,
            Alternatives::Odd(offsets) => &offsets.odd,
        }
    }

    fn other(self) -> Alternatives {
        match self {
            Alternatives::Even(offsets) => Alternatives::Odd(offsets),
            Alternatives::Odd(offsets) => Alternatives::Even(offsets),
        }
    }
}

fn coords_to_small_index_type(coords: Vec<(i8, i8)>) -> Vec<(SmallIndexType, SmallIndexType)> {
    let mut result = Vec::new();
    for (x, y) in coords {
        result.push(((x + 1) as SmallIndexType, (y + 1) as SmallIndexType));
    }
    result
}
