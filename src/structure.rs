use anyhow::Result;
use std::{
    fmt::{Debug, Display},
    hash::Hash,
};

pub trait Generation: Hash + Eq + PartialEq + Debug + Clone {
    fn successor(&self) -> Self;
}
pub trait Region<S: State<Gen>, Gen: Generation> {
    type Loc: Location<S, Gen>;

    fn locations(&self) -> impl IntoIterator<Item = Self::Loc>;
    fn state(&self, location: &Self::Loc, generation: &Gen) -> Option<S>;
}
pub trait Space<S: State<Gen>, Gen: Generation> {
    type Reg: Region<S, Gen>;

    fn regions(&self) -> impl IntoIterator<Item = Self::Reg>;

    fn reduce<A, F>(&self, init: A, f: F) -> A
    where
        F: Fn(&Self::Reg, &<Self::Reg as Region<S, Gen>>::Loc, A) -> A,
    {
        let mut accumulator = init;
        for region in self.regions() {
            for location in region.locations() {
                accumulator = f(&region, &location, accumulator);
            }
        }
        accumulator
    }
}
pub trait Location<S: State<Gen>, Gen: Generation>: Sized {
    fn effectors(&self) -> Result<impl IntoIterator<Item = Self>>;
    fn id(&self) -> String;
}
pub trait State<Gen: Generation>: Debug + Clone + Display {
    fn update<Reg: Region<Self, Gen>>(
        region: &Reg,
        location: &<Reg as Region<Self, Gen>>::Loc,
        generation: &Gen,
    ) -> Result<Self>;
}
pub trait GrayScale {
    type Context;
    fn gray_value(&self, context: &Self::Context) -> u8;
}

impl Generation for usize {
    fn successor(&self) -> Self {
        self + 1
    }
}
