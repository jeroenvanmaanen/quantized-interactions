use std::{collections::HashSet, fmt::Display};

use crate::{
    patch::new_hexagonal_torus,
    structure::{Generation, Location, Space, State},
    torus::Torus,
};

use anyhow::{Result, anyhow};
use log::info;

#[derive(Default, Debug, Clone, Copy)]
struct Trivial;

impl Display for Trivial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Trivial")
    }
}

impl<Spc, S, Gen> Location<Spc, S, Gen> for u8
where
    Spc: Space<S, Gen> + ?Sized,
    S: State<Gen>,
    Gen: Generation,
{
    fn effectors(&self, _space: &Spc) -> Result<impl IntoIterator<Item = Self>> {
        Ok(HashSet::new())
    }

    fn id(&self, _space: &Spc) -> String {
        format!("{}", &self)
    }
}

impl State<usize> for Trivial {
    fn update<Spc: Space<Self, usize>>(
        space: &Spc,
        _region: &Spc::Reg,
        location: &Spc::Loc,
    ) -> Result<Self> {
        let count = location.effectors(space)?.into_iter().count();
        if count != 6 && count != 0 {
            return Err(anyhow!("Wrong count: [{}]", count));
        }
        Ok(Trivial)
    }
}

pub fn example() -> Result<()> {
    info!("Patch PoC");
    let mut crystal = new_hexagonal_torus(Trivial::default(), 0usize, 40, 30)?;
    let generation = 0usize;
    crystal.info(&generation);
    crystal.update_all_cells(&generation)?;

    Ok(())
}
