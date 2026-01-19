use crate::{
    cell::{Cell, Generation, State},
    torus::{Tiling, Torus},
};
use anyhow::Result;
// use log::debug;
use log::{info, trace};
use std::f64::consts::PI;

#[derive(Clone, Debug)]
pub struct Rotate {
    pub angle: f64,
}

impl Rotate {
    pub fn new(angle: f64) -> Rotate {
        Rotate { angle }
    }
}

impl State for Rotate {
    type Gen = u32;

    fn update(cell: &Cell<Rotate>, generation: &u32) -> Result<Rotate> {
        trace!("Update: [{}]", cell.id());
        let this_state = cell.state(generation).map(|s| s.angle).unwrap_or(0.0);
        trace!("This state: [{this_state:?}]");
        let neighbors_lock = cell.neighbors()?;
        let mut count = 0;
        let mut angle = 0f64;
        for neighbor in neighbors_lock.iter() {
            count += 1;
            trace!("Neigbor: [{}]", neighbor.id());
            if let Some(state) = neighbor.state(generation) {
                angle += normalize(state.angle) + 2.0 * PI;
            }
        }
        let next_state = this_state + symmetric(angle / (count as f64)) / 12.0;
        let result = Rotate { angle: next_state };
        Ok(result)
    }

    fn to_char(&self) -> char {
        let a = normalize(self.angle);
        let u = PI / 3.0;
        if a < u {
            '-'
        } else if a < PI {
            '\\'
        } else if a < 5.0 * u {
            '/'
        } else if a < 7.0 * u {
            '-'
        } else if a < 3.0 * PI {
            '\\'
        } else if a < 11.0 * PI {
            '/'
        } else {
            '-'
        }
    }
}

fn normalize(angle: f64) -> f64 {
    if angle < 0.0 {
        angle + ((angle / (-2.0 * PI)).trunc() + 1.0) * 2.0 * PI
    } else if angle >= 2.0 * PI {
        angle + (angle / (2.0 * PI)).trunc() * 2.0 * PI
    } else {
        angle
    }
}

fn symmetric(angle: f64) -> f64 {
    let norm = normalize(angle);
    if norm <= PI { norm } else { norm - (2.0 * PI) }
}

pub fn example() -> Result<()> {
    let origin = Cell::new(0u32, Rotate::new(0.0));
    let dimensions = [5, 5, 5];
    let generation = 0u32;
    let torus = Torus::new(
        origin.clone(),
        Tiling::Orthogonal,
        &dimensions,
        generation.clone(),
        |v: &[usize]| Rotate::new(experiment_init(v, &dimensions)),
    )?;
    info!("Origin: [{origin:?}]");
    torus.info(&generation);
    torus.update_all(&0u32)?;
    let generation = generation.successor();
    torus.info(&generation);
    Ok(())
}

fn experiment_init(v: &[usize], dimensions: &[usize]) -> f64 {
    let dimensionality = v.len();
    let mut sum_of_squares = 0.0;
    let mut x = Vec::new();
    for d in 0..dimensionality {
        let offset = (dimensions[d] as f64) / 2.0;
        let x_d = (v[d] as f64) - offset;
        x.push(x_d);
        sum_of_squares += x_d * x_d;
    }
    let r = sum_of_squares.sqrt();
    let mut inner_product = 0.0;
    let diag = 1.0 / (dimensionality as f64).sqrt();
    for x_d in x {
        let xu_d = x_d / r;
        inner_product += xu_d * diag;
    }
    inner_product * PI
}
