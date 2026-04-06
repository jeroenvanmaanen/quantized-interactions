use crate::{
    cell::new_cell_torus,
    patch::new_hexagonal_torus,
    structure::{Generation, GrayScale, Location, Region, Space, State},
    torus::{GrayScaleTorus, Tiling, Torus, get_index},
};
use anyhow::Result;
use std::{
    cmp,
    f64::{MAX, consts::PI},
    fmt::{Display, Write},
    path::PathBuf,
};
// use log::debug;
use log::{info, trace};

#[derive(Copy, Clone, Debug, Default)]
pub struct Wave {
    amplitude: f64,
    velocity: f64,
    is_center: bool,
    effector_count: Option<u8>,
}

impl Wave {
    pub fn new(amplitude: f64, is_center: bool) -> Wave {
        Wave {
            amplitude,
            velocity: 0.0,
            is_center,
            effector_count: None,
        }
    }
}

impl State<usize> for Wave {
    fn update<Spc: Space<Self, usize>>(
        space: &Spc,
        region: &Spc::Reg,
        location: &Spc::Loc,
    ) -> Result<Self> {
        trace!("Update: [{}]", location.id());
        let this_state: Self = region.state(location).unwrap_or_default();
        trace!("This state: [{this_state:?}]");
        let effectors = location.effectors(space)?;
        let mut next_amplitude = this_state.amplitude;
        let mut next_velocity = this_state.velocity;
        next_amplitude += next_velocity;
        let mut count = 0;
        let mut err = 0;
        if this_state.is_center {
            let generation = region.generation();
            let angle = (generation as f64) / 40.0;
            next_amplitude = angle.sin() * 30.0;
            next_velocity = angle.cos();
            for _ in effectors {
                count += 1;
            }
        } else if let Some(this_c) = this_state.effector_count {
            for effector in effectors {
                trace!("Effector: [{}]", effector.id());
                if let Some(other_state) = region.state(&effector) as Option<Wave> {
                    if let Some(c) = other_state.effector_count {
                        let max_c = cmp::max(this_c, c);
                        let delta =
                            (other_state.amplitude - this_state.amplitude) * 0.005 / (max_c as f64);
                        next_velocity += delta;
                    }
                    count += 1;
                } else {
                    err += 1;
                }
            }
        } else {
            for _ in effectors {
                count += 1;
            }
        }
        trace!(
            "Effector count: {}: {} (err: {})",
            location.id(),
            count,
            err
        );
        let new_count = if count > 0 { Some(count) } else { None };
        let result = Wave {
            amplitude: next_amplitude,
            velocity: next_velocity,
            is_center: this_state.is_center,
            effector_count: new_count,
        };
        Ok(result)
    }
}

impl Display for Wave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // let s = format!("{:5.2}", self.amplitude);
        // f.write_str(&s)?;
        let c = if self.is_center {
            'o'
        } else if self.amplitude > 0.0 {
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

pub fn example(patched: bool, size: usize, export_dir: Option<&PathBuf>) -> Result<()> {
    if patched {
        patched_example(size, export_dir)?
    } else {
        cell_example(size, export_dir)?
    }
    Ok(())
}

fn patched_example(size: usize, export_dir: Option<&PathBuf>) -> Result<()> {
    let width = size;
    let height = size;
    let generation = 0usize;

    let init = Wave::new(0.0, false);
    let mut torus = new_hexagonal_torus(init, generation.clone(), width, height)?;

    let center = Wave::new(0.0, true);
    torus.adjust(&generation, size / 2, size / 2, center)?;

    run_example(torus, generation, export_dir)
}

fn cell_example(size: usize, export_dir: Option<&PathBuf>) -> Result<()> {
    let width = size;
    let height = size;
    let generation = 0usize;

    let torus = new_cell_torus(
        Tiling::Hexagons,
        &[height, width],
        generation.clone(),
        |v: &[usize]| {
            let c = v[0] / 2 == height / 4 && v[1] / 2 == width / 4;
            Wave::new(0.0, c)
        },
    )?;

    run_example(torus, generation, export_dir)
}

fn run_example<T: Torus<Wave, usize> + GrayScaleTorus<Wave, usize>>(
    torus: T,
    generation: usize,
    export_dir: Option<&PathBuf>,
) -> Result<()> {
    let mut generation = generation;
    let size = torus.dimensions()[0];
    let mut torus = torus;
    // torus.info(&generation);
    for i in 1..=(size * 10) {
        torus.space_mut().update_all(&generation)?;
        generation = generation.successor();
        // torus.info(&generation);
        let m = smallest_local_maximum(torus.space(), &generation);
        info!("Smallest local maximum: [{generation}]: [{m}]");
        if i % size == 0 {
            torus.export(&generation, &m, export_dir)?;
        }
    }
    Ok(())
}

#[derive(Default, Debug, Clone)]
struct Coords(usize, usize, usize);

impl<Gen: Generation> State<Gen> for Coords {
    fn update<Spc: Space<Self, Gen>>(
        _space: &Spc,
        region: &Spc::Reg,
        location: &Spc::Loc,
    ) -> Result<Self> {
        return Ok(region.state(location).unwrap_or_default());
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
    let width = size;
    let height = size;
    let dimensions = [height, width];
    let generation = 0usize;
    let torus = new_cell_torus(
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

fn smallest_local_maximum(torus: &impl Space<Wave, usize>, generation: &usize) -> f64 {
    let result = torus.reduce(generation, MAX, |r, c, a| {
        if let Ok(Some(amplitude)) = local_maximum(torus, r, c) {
            if amplitude < a { amplitude } else { a }
        } else {
            a
        }
    });
    if result <= 0.0 { 1.0 } else { result }
}

fn local_maximum<Spc: Space<Wave, usize>>(
    space: &Spc,
    region: &Spc::Reg,
    location: &Spc::Loc,
) -> Result<Option<f64>> {
    if let Some(this_state) = region.state(location) as Option<Wave> {
        let amplitude = this_state.amplitude.abs();
        if amplitude <= 0.0 {
            return Ok(None);
        }
        for effector in location.effectors(space)? {
            if let Some(other_state) = region.state(&effector) as Option<Wave> {
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
