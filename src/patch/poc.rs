use std::{collections::HashSet, fmt::Display};

use crate::{
    patch::{AtMostSixEffectors, Inflexible, Patch},
    structure::{Location, Region, State},
};

use anyhow::Result;
use log::info;

#[derive(Default, Debug, Clone, Copy)]
struct Trivial;

impl Display for Trivial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Trivial")
    }
}

impl Location<Trivial, usize> for u8 {
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
        _location: &<Reg as Region<Self, usize>>::Loc,
        _generation: &usize,
    ) -> Result<Self> {
        Ok(Trivial)
    }
}

pub fn example() -> Result<()> {
    info!("Patch PoC");
    let effectors = AtMostSixEffectors::default();
    let _patch = Patch::new_init(Trivial::default());
    let circumference = 30;
    let capacity = circumference * circumference;
    let generation = 0usize;
    let _inflexible = Inflexible::new(effectors, capacity, &generation, Trivial::default());

    Ok(())
}
