use std::{collections::HashSet, fmt::Display};

use crate::{
    cell::{Location, Region, State},
    patch::{AtMostSixNeighbors, Patch},
};

use anyhow::Result;

#[derive(Default, Debug, Clone, Copy)]
struct Trivial;

impl Display for Trivial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Trivial")
    }
}

impl Location<Trivial, usize> for u8 {
    fn neighbors(&self) -> Result<impl IntoIterator<Item = Self>> {
        Ok(HashSet::new())
    }

    fn id(&self) -> String {
        format!("{}", &self)
    }
}

impl Region<Trivial, usize> for () {
    type Loc = u8;
    fn state(&self, _location: &Self::Loc, _generation: &usize) -> Option<Trivial> {
        None
    }
}

impl State<usize> for Trivial {
    fn update<Reg: Region<Self, usize>>(
        _region: &Reg,
        _location: &<Reg as Region<Self, usize>>::Loc,
        _generation: &usize,
    ) -> Result<Self> {
        Ok(Trivial)
    }
}

pub fn example() -> Result<()> {
    let neighbors = AtMostSixNeighbors::default();
    let _patch = Patch::new_init(neighbors, Trivial::default());
    todo!()
}
