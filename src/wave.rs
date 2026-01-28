use std::{
    fmt::{Display, Write},
    path::PathBuf,
};

use crate::{
    cell::{Cell, Generation, GrayScale, State},
    torus::{Tiling, Torus, get_index},
};
use anyhow::Result;
// use log::debug;
use log::{info, trace};

#[derive(Clone, Debug, Default)]
pub struct Wave {
    pub amplitude: f64,
    force: f64,
    neighbor_count: Option<u8>,
}

impl Wave {
    pub fn new(amplitude: f64) -> Wave {
        Wave {
            amplitude,
            force: 0.0,
            neighbor_count: None,
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
                if let Some(c) = state.neighbor_count {
                    next_amplitude += state.force / (c as f64);
                }
                next_force += this_state.amplitude - state.amplitude;
                count += 1;
            } else {
                err += 1;
            }
        }
        trace!("Neighbor count: {}: {} (err: {})", cell.id(), count, err);
        next_force *= 0.04;
        let result = Wave {
            amplitude: next_amplitude,
            force: next_force,
            neighbor_count: Some(count),
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

impl GrayScale for Wave {
    type Context = f64;

    fn gray_value(&self, context: &f64) -> u8 {
        let magnitude = self.amplitude.abs().sqrt();
        let sign = self.amplitude.signum();
        ((127.0 * sign * magnitude / context) + 128.0) as u8
    }
}

pub fn example(size: usize, export_dir: Option<&PathBuf>) -> Result<()> {
    let origin = Cell::new(0usize, Wave::new(0.0));
    info!("Origin: [{origin:?}]");
    let width = size;
    let height = size;
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
    for _ in 1..(size / 2) {
        torus.update_all(&generation)?;
        generation = generation.successor();
        torus.info(&generation);
        let m = max_amplitude(&torus, &generation).abs().sqrt();
        torus.export(&generation, &m, export_dir)?;
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

pub fn debug(size: usize) -> Result<()> {
    let origin = Cell::new(0usize, Coords(0, 0, 0));
    info!("Origin: [{origin:?}]");
    let width = size;
    let height = size;
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

fn max_amplitude(torus: &Torus<Wave>, generation: &<Wave as State>::Gen) -> f64 {
    let mut result = 0.0f64;
    torus.reduce(&mut result, |c, a| {
        let v = c
            .state(generation)
            .map(|s| s.amplitude.abs())
            .unwrap_or(0.0);
        if v > *a {
            *a = v;
        }
    });
    if result <= 0.0 {
        result = 1.0;
    }
    result
}
