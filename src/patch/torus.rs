use std::rc::Rc;

use anyhow::{Result, anyhow};
use log::debug;

use crate::{
    patch::{AtMostSixEffectors, Effectors, PATCH_SIZE},
    structure::{Generation, State},
    torus::{Tiling, Torus},
};

use super::Crystal;

pub struct PatchTorus<S: State<Gen> + Copy, Gen: Generation, E: Effectors> {
    tiling: Tiling,
    dimensions: Vec<usize>,
    crystal: Crystal<S, Gen, E>,
}

impl<S: State<Gen> + Copy, Gen: Generation, E: Effectors> Torus<S, Gen> for PatchTorus<S, Gen, E> {
    fn update_all(&self, generation: &Gen) -> Result<()> {
        self.crystal.update_all(generation)
    }

    fn info(&self, _generation: &Gen) {
        todo!()
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
    connect_cells(&mut crystal, width, w, height, h)?;
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

fn connect_cells<S, Gen, E>(
    crystal: &mut Crystal<S, Gen, E>,
    width: usize,
    w: usize,
    height: usize,
    h: usize,
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

    let wp = width / w; // With of a small patch
    let wq = width - w * (wp as usize); // Number of collums that are one cell wider
    let wp = wp as u8;
    let w_wrap = w <= 1;
    let hp = height / h; // Height of a small patch
    let hq = height - h * hp; // Number of rows that are one cell taller
    let hp = hp as u8;
    let h_wrap = h <= 1;
    let mut cell_rows_before = 0;
    let mut br = 0;
    for r in 0..h {
        let hr = hp + (if r < hq { 1 } else { 0 }); // Height of this row
        let mut cell_colums_before = 0;
        for c in 0..w {
            let wc = wp + (if c < wq { 1 } else { 0 }); // Width of this column
            let even = (cell_rows_before ^ cell_colums_before) & 0x01 == 0; // TODO: is this correct?
            let p = br + c;
            debug!(
                "Patch: #{p}: [{r}]: [{c}]: ([{wc}] x [{hr}]): [{cell_colums_before}, {cell_rows_before}, {even}]"
            );
            let effectors = &mut crystal.effectors[p];
            let mut offsets = even_offsets.clone();
            if !even {
                offsets = offsets.other();
            }
            let mut iy = wc;
            for _ in 1..(hr - 1) {
                for x in 1..(wc - 1) {
                    let i = iy + x;
                    for (ox, oy) in offsets.offsets() {
                        let j = i + (oy * wc) - wc + ox - 1;
                        effectors.add(i, j)?;
                    }
                }
                offsets = offsets.other();
                iy += wc;
            }
            if w_wrap {
                debug!("Wrap left to right");
            }
            if h_wrap {
                debug!("Wrap top to bottom");
            }
            cell_colums_before += wc;
        }
        cell_rows_before += hr;
        br += w;
    }
    Ok(())
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
