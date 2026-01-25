use std::fmt::{Display, Write};

use crate::{
    cell::{Cell, Generation, State},
    torus::{Tiling, Torus, get_index},
};
use anyhow::Result;
// use log::debug;
use log::{info, trace};

#[derive(Clone, Debug, Default)]
pub struct Wave {
    pub amplitude: f64,
    force: f64,
}

impl Wave {
    pub fn new(amplitude: f64) -> Wave {
        Wave {
            amplitude,
            force: 0.0,
        }
    }
}

impl State for Wave {
    type Gen = usize;

    fn update(cell: &Cell<Wave>, generation: &usize) -> Result<Wave> {
        trace!("Update: [{}]", cell.id());
        let this_state = cell.state(generation).unwrap_or_default();
        trace!("This state: [{this_state:?}]");
        let neighbors_lock = cell.neighbors()?;
        let mut next_amplitude = this_state.amplitude - this_state.force;
        let mut next_force = 0.0;
        let mut count = 0;
        let mut err = 0;
        for neighbor in neighbors_lock.iter() {
            trace!("Neigbor: [{}]", neighbor.id());
            if let Some(state) = neighbor.state(generation) {
                next_amplitude += state.force / 6.0;
                next_force += this_state.amplitude - state.amplitude;
                count += 1;
            } else {
                err += 1;
            }
        }
        trace!("Neighbor count: {}: {} (err: {})", cell.id(), count, err);
        next_force *= 0.1;
        let result = Wave {
            amplitude: next_amplitude,
            force: next_force,
        };
        Ok(result)
    }
}

impl Display for Wave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // let s = format!("{:5.2}", self.amplitude);
        // f.write_str(&s)?;
        let c = if self.amplitude > 0.0 {
            '^'
        } else if self.amplitude < 0.0 {
            'v'
        } else {
            '0'
        };
        f.write_char(c)?;
        Ok(())
    }
}

pub fn example() -> Result<()> {
    let origin = Cell::new(0usize, Wave::new(0.0));
    info!("Origin: [{origin:?}]");
    let width = 34;
    let height = 34;
    let mut generation = 0usize;
    let torus = Torus::new(
        origin.clone(),
        Tiling::Hexagons,
        &[height, width],
        generation.clone(),
        |v: &[usize]| {
            Wave::new(if v[0] == height / 2 && v[1] == width / 2 {
                1000.0
            } else {
                0.0
            })
        },
    )?;
    info!("Origin: [{origin:?}]");
    torus.info(&generation);
    for _ in 1..17 {
        torus.update_all(&generation)?;
        generation = generation.successor();
        torus.info(&generation);
    }
    Ok(())
}

#[derive(Default, Debug, Clone)]
struct Coords(usize, usize, usize);

impl State for Coords {
    type Gen = usize;

    fn update(cell: &Cell<Coords>, generation: &usize) -> Result<Coords> {
        return Ok(cell.state(generation).unwrap_or_default());
    }
}

impl Display for Coords {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = format!("({}, {})#{}", self.0, self.1, self.2);
        f.write_str(&s)?;
        Ok(())
    }
}

#[allow(dead_code)]
pub fn debug() -> Result<()> {
    let origin = Cell::new(0usize, Coords(0, 0, 0));
    info!("Origin: [{origin:?}]");
    let width = 4;
    let height = 4;
    let dimensions = [height, width];
    let generation = 0usize;
    let torus = Torus::new(
        origin.clone(),
        Tiling::Hexagons,
        &dimensions,
        generation.clone(),
        |v: &[usize]| {
            Coords(
                v[0],
                v[1],
                match get_index(v, &dimensions) {
                    Ok(i) => i,
                    _ => 0,
                },
            )
        },
    )?;
    torus.info(&generation);
    Ok(())
}
