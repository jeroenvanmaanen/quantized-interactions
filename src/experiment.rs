use crate::{
    cell::{Cell, Generation, State},
    torus::{Tiling, Torus},
};
use anyhow::Result;
// use log::debug;
use log::{info, trace};
use std::f64::consts::{PI, SQRT_2};

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
    let width = 5;
    let height = 5;
    let generation = 0u32;
    let torus = Torus::new(
        origin.clone(),
        Tiling::TouchingSquares,
        width,
        height,
        generation.clone(),
        |v: &[usize]| Rotate::new(experiment_init(v[0], v[1])),
    )?;
    info!("Origin: [{origin:?}]");
    torus.info(&generation);
    torus.update_all(&0u32)?;
    let generation = generation.successor();
    torus.info(&generation);
    Ok(())
}

fn experiment_init(x: usize, y: usize) -> f64 {
    let xx = (x as f64) - 2.0;
    let yy = (y as f64) - 2.0;
    let rr = euclidean_length(xx, yy);
    let xu = xx * rr;
    let yu = yy * rr;
    ((xu * SQRT_2) + (yu * SQRT_2)) * PI
}

fn euclidean_length(x: f64, y: f64) -> f64 {
    ((x * x) + (y * y)).sqrt()
}
