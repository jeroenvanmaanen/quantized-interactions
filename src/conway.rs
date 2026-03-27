use std::fmt::{Display, Write};

use crate::{
    cell::new_cell_torus,
    structure::{Generation, Location, Region, Space, State},
    torus::{Tiling, Torus},
};
use anyhow::Result;
// use log::debug;
// use log::info;
use log::trace;

#[derive(Clone, Debug)]
pub struct Conway {
    pub alive: bool,
}

impl Conway {
    pub fn new(alive: bool) -> Conway {
        Conway { alive }
    }
}

impl State<usize> for Conway {
    fn update<Spc: Space<Self, usize>>(
        space: &Spc,
        region: &Spc::Reg,
        location: &Spc::Loc,
    ) -> Result<Self> {
        trace!("Update: [{}]", location.id());
        let this_state = (region.state(location) as Option<Self>)
            .map(|s| s.alive)
            .unwrap_or(false);
        trace!("This state: [{this_state:?}]");
        let mut count = 0;
        for effector in location.effectors(space)? {
            trace!("Effector: [{}]", effector.id());
            if let Some(state) = region.state(&effector) as Option<Self> {
                if state.alive {
                    count += 1;
                }
            }
        }
        let next_state = count == 3 || (this_state && count == 2);
        let result = Conway { alive: next_state };
        Ok(result)
    }
}

impl Display for Conway {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(if self.alive { '#' } else { ' ' })?;
        Ok(())
    }
}

pub fn example() -> Result<()> {
    let width = 5;
    let height = 5;
    let generation = 0usize;
    let mut torus = new_cell_torus(
        Tiling::OrthogonalAndDiagonal,
        &[width, height],
        generation.clone(),
        |v: &[usize]| Conway::new(v[1] == 2 && (v[0] >= 1 && v[0] <= 3)),
    )?;
    torus.info(&generation);
    torus.update_all(&0usize)?;
    let generation = generation.successor();
    torus.info(&generation);
    Ok(())
}
