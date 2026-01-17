use crate::cell::{Cell, State};
use anyhow::Result;
// use log::debug;
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

impl State for Conway {
    type Gen = u32;

    fn update(cell: &Cell<Conway>, generation: &u32) -> Result<Conway> {
        trace!("Update: [{}]", cell.id());
        let this_state = cell.state(generation).map(|s| s.alive).unwrap_or(false);
        trace!("This state: [{this_state:?}]");
        let neighbors_lock = cell.neighbors()?;
        let mut count = 0;
        for neighbor in neighbors_lock.iter() {
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

    fn to_char(&self) -> char {
        if self.alive { '#' } else { ' ' }
    }
}
