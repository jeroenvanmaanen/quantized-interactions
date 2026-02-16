use std::fmt::{Display, Write};

use crate::{
    cell::{Cell, Generation, Location, State},
    torus::{Tiling, Torus},
};
use anyhow::Result;
// use log::debug;
use log::{info, trace};

#[derive(Clone, Debug)]
pub struct Conway {
    pub alive: bool,
}

impl Conway {
    pub fn new(alive: bool) -> Conway {
        Conway { alive }
    }
}

impl State for Conway {
    type Gen = usize;
    type Loc = Cell<Self>;

    fn update(cell: &Self::Loc, generation: &usize) -> Result<Conway> {
        trace!("Update: [{}]", cell.id());
        let this_state = cell.state(generation).map(|s| s.alive).unwrap_or(false);
        trace!("This state: [{this_state:?}]");
        let mut count = 0;
        for neighbor in cell.neighbors()? {
            trace!("Neigbor: [{}]", neighbor.id());
            if let Some(state) = neighbor.state(generation) {
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
    let origin = Cell::new(0usize, Conway::new(false));
    let other = Cell::new(1usize, Conway::new(true));
    origin.join(&other)?;
    info!("Origin: [{origin:?}]");
    let width = 5;
    let height = 5;
    let generation = 0usize;
    let torus = Torus::new(
        origin.clone(),
        Tiling::OrthogonalAndDiagonal,
        &[width, height],
        generation.clone(),
        |v: &[usize]| Conway::new(v[1] == 2 && (v[0] >= 1 && v[0] <= 3)),
    )?;
    info!("Origin: [{origin:?}]");
    torus.info(&generation);
    torus.update_all(&0usize)?;
    let generation = generation.successor();
    torus.info(&generation);
    Ok(())
}
