use std::{collections::HashSet, fmt::Display};

use crate::{
    patch::new_hexagonal_torus,
    structure::{Location, Region, State},
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

impl Location<usize> for u8 {
    fn effectors(&self) -> Result<impl IntoIterator<Item = Self>> {
        Ok(HashSet::new())
    }

    fn id(&self) -> String {
        format!("{}", &self)
    }
}

impl Region<Trivial, usize> for () {
    type Loc = u8;

    fn locations(&self) -> impl IntoIterator<Item = Self::Loc> {
        HashSet::new()
    }

    fn state(&self, _location: &Self::Loc, _generation: &usize) -> Option<Trivial> {
        None
    }
}

impl State<usize> for Trivial {
    fn update<Reg: Region<Self, usize>>(
        _region: &Reg,
        location: &<Reg as Region<Self, usize>>::Loc,
        _generation: &usize,
    ) -> Result<Self> {
        let count = location.effectors().into_iter().count();
        if count != 6 {
            return Err(anyhow!("Wrong count: [{}]", count));
        }
        Ok(Trivial)
    }
}

pub fn example() -> Result<()> {
    info!("Patch PoC");
    let _crystal = new_hexagonal_torus(Trivial::default(), 0usize, 40, 30);
    // TODO:    _crystal.update_all();

    Ok(())
}
