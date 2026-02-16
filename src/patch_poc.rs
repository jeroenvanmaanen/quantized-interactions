use std::{collections::HashSet, fmt::Display};

use crate::{
    cell::{Cell, Location, Region, State},
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

impl Location<Trivial> for u8 {
    fn neighbors(&self) -> Result<impl IntoIterator<Item = Self>> {
        Ok(HashSet::new())
    }

    fn id(&self) -> String {
        format!("{}", &self)
    }
}

impl Region<Trivial> for () {
    fn state(&self, _: &<Trivial as State>::Loc, _: &<Trivial as State>::Gen) -> Option<Trivial> {
        None
    }
}

impl State for Trivial {
    type Gen = usize;
    type Reg = ();
    type Loc = Cell<Trivial>;

    fn update(_: &(), _cell: &Self::Loc, _generation: &Self::Gen) -> Result<Self> {
        Ok(Trivial)
    }
}

pub fn example() -> Result<()> {
    let neighbors = AtMostSixNeighbors::default();
    let _patch = Patch::new_init(neighbors, Trivial::default());
    todo!()
}
