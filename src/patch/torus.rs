use anyhow::{Result, anyhow};
use log::debug;

use crate::{
    patch::{AtMostSixEffectors, Effectors, PATCH_SIZE},
    structure::{Generation, State},
    torus::Tiling,
};

use super::Crystal;

pub struct PatchTorus<S: State<Gen> + Copy, Gen: Generation, E: Effectors> {
    tiling: Tiling,
    dimensions: Vec<usize>,
    crystal: Crystal<S, Gen, E>,
}

const MAX_PATCH_SIZE: u8 = 14 * 18;
const SQRT_PATCH_SIZE: u8 = 17;

pub fn new_hexagonal<S: State<Gen> + Copy, Gen: Generation>(
    init: S,
    initial_gen: Gen,
    width: usize,
    height: usize,
) -> Result<PatchTorus<S, Gen, AtMostSixEffectors>> {
    if width % 2 == 1 || height % 2 == 1 {
        return Err(anyhow!("Must both be even: ({width}, {height})"));
    }
    let dimensions = vec![width, height];
    let effectors = AtMostSixEffectors::default();
    let (w, h) = calculate_grid(width, height);
    let crystal = Crystal::new(effectors, w * h, &initial_gen, init);
    connect_cells(&crystal, width, w, height, h);
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
    let sd = if s > 1 { 1 } else { 0 };
    let sx = sd + (short + s - 1) / s;
    let lx = (PATCH_SIZE as usize) / sx;
    let ld = if lx < long { 1 } else { 0 };
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
    crystal: &Crystal<S, Gen, E>,
    width: usize,
    w: usize,
    height: usize,
    h: usize,
) where
    S: State<Gen> + Copy,
    Gen: Generation,
    E: Effectors,
{
    debug!(
        "Connect cells: [{}]: ([{width}] / [{w}]) x ([{height}] / [{h}])",
        crystal.patch_count()
    );
    let wp = width / w; // With of a small patch
    let wq = width - w * wp; // Number of collums that are one cell wider
    let hp = height / h; // Height of a small patch
    let hq = height - h * hp; // Number of rows that are one cell taller
    for r in 0..h {
        let hr = hp + (if r < hq { 1 } else { 0 }); // Height of this row
        for c in 0..w {
            let wc = wp + (if c < wq { 1 } else { 0 }); // Width of this column
            debug!("Patch: [{r}]: [{c}]: ([{wc}] x [{hr}])");
        }
    }
}
