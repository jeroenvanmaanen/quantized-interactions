use std::{
    cmp,
    f64::{MAX, consts::PI},
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
    is_center: bool,
    neighbor_count: Option<u8>,
}

impl Wave {
    pub fn new(amplitude: f64, is_center: bool) -> Wave {
        Wave {
            amplitude,
            is_center,
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
        let mut next_amplitude = this_state.amplitude;
        let mut count = 0;
        let mut err = 0;
        if this_state.is_center {
            next_amplitude = ((*generation as f64) / 10.0).sin() * 30.0;
            count = neighbors_lock.len() as u8;
        } else if let Some(this_c) = this_state.neighbor_count {
            for neighbor in neighbors_lock.iter() {
                trace!("Neigbor: [{}]", neighbor.id());
                if let Some(other_state) = neighbor.state(generation) {
                    if let Some(c) = other_state.neighbor_count {
                        let min_c = cmp::min(this_c, c);
                        let delta =
                            (other_state.amplitude - this_state.amplitude) * 0.3 / (min_c as f64);
                        next_amplitude += delta;
                    }
                    count += 1;
                } else {
                    err += 1;
                }
            }
        } else {
            count = neighbors_lock.len() as u8;
        }
        trace!("Neighbor count: {}: {} (err: {})", cell.id(), count, err);
        let new_count = if count > 0 { Some(count) } else { None };
        let result = Wave {
            amplitude: next_amplitude,
            is_center: this_state.is_center,
            neighbor_count: new_count,
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

    fn gray_value(&self, smallest_local_maximum: &f64) -> u8 {
        let magnitude = self.amplitude / smallest_local_maximum;
        let value = magnitude.atan() * 2.0 / PI;
        ((127.0 * value) + 128.0) as u8
    }
}

pub fn example(size: usize, export_dir: Option<&PathBuf>) -> Result<()> {
    let origin = Cell::new(0usize, Wave::new(0.0, false));
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
            let c = v[0] / 2 == height / 4 && v[1] / 2 == width / 4;
            Wave::new(0.0, c)
        },
    )?;
    info!("Origin: [{origin:?}]");
    // torus.info(&generation);
    for _ in 1..(size * 2) {
        torus.update_all(&generation)?;
        generation = generation.successor();
        // torus.info(&generation);
        let m = smallest_local_maximum(&torus, &generation);
        info!("Smallest local maximum: [{generation}]: [{m}]");
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

fn smallest_local_maximum(torus: &Torus<Wave>, generation: &<Wave as State>::Gen) -> f64 {
    let mut result = MAX;
    torus.reduce(&mut result, |c, a| {
        if let Ok(Some(amplitude)) = local_maximum(c, generation) {
            if amplitude < *a {
                *a = amplitude;
            }
        }
    });
    if result <= 0.0 {
        result = 1.0;
    }
    result
}

fn local_maximum(cell: &Cell<Wave>, generation: &<Wave as State>::Gen) -> Result<Option<f64>> {
    if let Some(this_state) = cell.state(generation) {
        let amplitude = this_state.amplitude.abs();
        if amplitude <= 0.0 {
            return Ok(None);
        }
        for neighbor in cell.neighbors()?.iter() {
            if let Some(other_state) = neighbor.state(generation) {
                if other_state.amplitude.abs() > amplitude {
                    return Ok(None);
                }
            }
        }
        Ok(Some(amplitude))
    } else {
        Ok(None)
    }
}
