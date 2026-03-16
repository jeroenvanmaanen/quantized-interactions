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
    // TODO: connect effectors of all cells
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
    let sm = (short + 17) / 18;
    let mut s = sm;
    let (mut l, mut q) = calculate_footprint(long, short, sm);
    if s > 1 {
        let (la, qa) = calculate_footprint(long, short, sm - 1);
        if qa < q {
            s = sm - 1;
            l = la;
            q = qa;
        }
    }
    let (lb, qb) = calculate_footprint(long, short, sm + 1);
    if qb < q { (lb, sm + 1) } else { (l, s) }
}

fn calculate_footprint(long: usize, short: usize, s: usize) -> (usize, usize) {
    let sx = (short + s - 1) / s;
    let lx = (PATCH_SIZE as usize + sx - 1) / sx;
    let l = (long + lx - 1) / lx;
    let footprint = (l, s * l);
    debug!(
        "Footprint: ({long} / {l}, {short} / {s}) -> ({lx}, {sx}) -> {}",
        footprint.1
    );
    footprint
}
