mod info;

use anyhow::{Result, anyhow};
use log::{debug, warn};
use std::rc::Rc;

use crate::{
    patch::{AtMostSixEffectors, Effectors, PATCH_SIZE},
    structure::{Generation, State},
    torus::{Tiling, Torus},
};
use info::info_hexagons;

use super::Crystal;

pub struct PatchTorus<S: State<Gen> + Copy, Gen: Generation, E: Effectors> {
    tiling: Tiling,
    dimensions: Vec<usize>,
    crystal: Crystal<S, Gen, E>,
}

impl<S: State<Gen> + Copy, Gen: Generation> Torus<S, Gen>
    for PatchTorus<S, Gen, AtMostSixEffectors>
{
    fn update_all(&self, generation: &Gen) -> Result<()> {
        self.crystal.update_all(generation)
    }

    fn info(&self, _generation: &Gen) {
        if self.tiling == Tiling::Hexagons {
            info_hexagons(self);
        } else {
            warn!("No info for tiling: [{:?}]", self.tiling)
        }
    }
}

const MAX_PATCH_SIZE: u8 = 14 * 18;
const SQRT_PATCH_SIZE: u8 = 17;

pub fn new_hexagonal_torus<S: State<Gen> + Copy, Gen: Generation>(
    init: S,
    initial_gen: Gen,
    width: usize,
    height: usize,
) -> Result<PatchTorus<S, Gen, AtMostSixEffectors>> {
    if width % 2 == 1 || height % 2 == 1 {
        return Err(anyhow!("Must both be even: ({width}, {height})"));
    }
    let dimensions = vec![width, height];
    let effector_factory = || AtMostSixEffectors::default();
    let (w, h) = calculate_grid(width, height);
    let mut crystal = Crystal::new(w * h, &initial_gen, init, effector_factory);
    connect_cells_hexagonally(&mut crystal, width, w, height, h, &initial_gen)?;
    Ok(PatchTorus {
        crystal,
        dimensions,
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
    let lx = (PATCH_SIZE as usize) / sx;
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

fn connect_cells_hexagonally<S, Gen, E>(
    crystal: &mut Crystal<S, Gen, E>,
    width: usize,
    w: usize,
    height: usize,
    h: usize,
    generation: &Gen,
) -> Result<()>
where
    S: State<Gen> + Copy,
    Gen: Generation,
    E: Effectors,
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
            let even = (cell_rows_before ^ cell_colums_before) & 0x01 == 0; // TODO: is this correct?
            crystal.patch_links[p].width = wc;
            crystal.patch_links[p].height = hr;
            crystal.patch_links[p].even = even;
            debug!(
                "Patch: #{p}: [{r}]: [{c}]: ([{wc}] x [{hr}]): [{cell_colums_before}, {cell_rows_before}, {even}]"
            );
            if let Some(patches) = crystal.generations.get_mut(generation) {
                if let Some(patch) = patches.get_mut(p) {
                    patch.borrow_mut().size = wc * hr;
                }
            }

            let effectors = &mut crystal.patch_links[p].effectors;
            let mut offsets = even_offsets.clone();
            if !even {
                offsets = offsets.other();
            }
            let er = if h > 1 { 1 } else { 0 };
            let mut iy = 0;
            for y in er..(hr - er) {
                let ec = if w > 1 { 1 } else { 0 };
                for x in ec..(wc - ec) {
                    let i = iy + x;
                    for (ox, oy) in offsets.offsets() {
                        let xx = (x + ox + wc - 1) % wc;
                        let yy = (y + oy + hr - 1) % hr;
                        let j = (yy * wc) + xx;
                        effectors.add(i, j)?;
                    }
                }
                offsets = offsets.other();
                iy += wc;
            }
            effectors.debug(format!("{p}"));
            cell_colums_before += wc;

            let edges = &mut crystal.patch_links[p].edges;
            if h > 1 {
                let this_base = (hr - 1) * wc;
                let above = (r + h - 1) % h;
                let above_base = (patch_grid.row_height(above) - 2) * wc;
                let below = (r + 1) % h;
                let fudge = if w > 1 { 1 } else { 0 };
                for i in fudge..(wc - fudge) {
                    edges.insert(i, (above * w + c, above_base + i)); // top to bottom of above
                    edges.insert(this_base + i, (below * w + c, wc + i)); // bottom to top of below
                }
            }
            if w > 1 {
                let row = r * w;
                let this_base = wc - 1;
                let left = (c + w - 1) % w;
                let left_width = patch_grid.column_width(left);
                let right = (c + 1) % w;
                let right_base = patch_grid.column_width(right);
                let fudge = if h > 1 { 1 } else { 0 };
                for i in fudge..(hr - fudge) {
                    edges.insert(i * wc, (row + left, ((i + 1) * left_width) - 2)); // leftmost cell to rightmost cell of patch to the left
                    edges.insert(this_base + (i * wc), (row + right, (i * right_base) + 1)); // rightmost cell to leftmost cell of patch to the right
                }
                if h > 1 {
                    // Insert corners
                    let above = (r + h - 1) % h;
                    let lft = (c + w - 1) % w;
                    let lft_wc = patch_grid.column_width(lft);
                    let right = (c + 1) % w;
                    let below = (r + 1) % h;

                    let top_lft_remote_index = patch_grid.bot_row_base(above, lft) + lft_wc - 2;
                    edges.insert(0, (above * w + lft, top_lft_remote_index));
                    let top_right_remote_index = patch_grid.bot_row_base(above, right) + 1;
                    edges.insert(wc - 1, (above * w + right, top_right_remote_index));
                    let bot_lft_remote_index = patch_grid.top_row_base(c) + lft_wc - 2;
                    edges.insert((hr - 1) * wc, (below * w + lft, bot_lft_remote_index));
                    let bot_right_remote_index = patch_grid.top_row_base(c) + 1;
                    edges.insert(hr * wc - 1, (below * w + right, bot_right_remote_index));
                }
            }
        }
        cell_rows_before += hr;
        br += w;
    }
    Ok(())
}

struct PatchGrid {
    w: usize,  // Number of patches horizontally
    wp: u8,    // Base width of patches
    wq: usize, // Number of patch columns that are one cell wider
    h: usize,  // Number of patches vertically
    hp: u8,    // Base height of patches
    hq: usize, // Number of patch rows that are one cell wider
}

impl PatchGrid {
    fn new(width: usize, w: usize, height: usize, h: usize) -> Self {
        let mut wp = width / w; // With of a small patch
        let wq = width - w * (wp as usize); // Number of collums that are one cell wider
        if w > 1 {
            wp += 2; // Add width for edges
        }
        let wp = wp as u8;
        let mut hp = height / h + (if h > 1 { 2 } else { 0 }); // Height of a small patch
        let hq = height - h * hp; // Number of rows that are one cell taller
        if h > 1 {
            hp += 2; // Add height for edges
        }
        let hp = hp as u8;
        PatchGrid {
            w,
            wp,
            wq,
            h,
            hp,
            hq,
        }
    }

    fn row_height(&self, r: usize) -> u8 {
        self.hp + (if r < self.hq { 1 } else { 0 })
    }

    fn column_width(&self, c: usize) -> u8 {
        self.wp + (if c < self.wq { 1 } else { 0 })
    }

    fn top_row_base(&self, c: usize) -> u8 {
        self.column_width(c)
    }

    fn bot_row_base(&self, r: usize, c: usize) -> u8 {
        let hr = self.row_height(r);
        let wc = self.column_width(c);
        (hr - 2) * wc
    }
}

struct Offsets {
    even: Vec<(u8, u8)>,
    odd: Vec<(u8, u8)>,
}

#[derive(Clone)]
enum Alternatives {
    Even(Rc<Offsets>),
    Odd(Rc<Offsets>),
}

impl Alternatives {
    fn new(even_coords: Vec<(i8, i8)>, odd_coords: Vec<(i8, i8)>) -> Self {
        let even = coords_to_u8(even_coords);
        let odd = coords_to_u8(odd_coords);
        Alternatives::Even(Rc::new(Offsets { even, odd }))
    }

    fn offsets(&self) -> &[(u8, u8)] {
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

fn coords_to_u8(coords: Vec<(i8, i8)>) -> Vec<(u8, u8)> {
    let mut result = Vec::new();
    for (x, y) in coords {
        result.push(((x + 1) as u8, (y + 1) as u8));
    }
    result
}
