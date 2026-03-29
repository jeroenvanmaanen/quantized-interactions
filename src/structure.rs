use anyhow::Result;
use std::{
    fmt::{Debug, Display},
    hash::Hash,
};

pub trait Generation: Hash + Eq + PartialEq + Debug + Clone {
    fn successor(&self) -> Self;
}

pub trait Region<Spc: Space<S, Gen> + ?Sized, S: State<Gen>, Gen: Generation>: Sized {
    fn locations(&self) -> impl IntoIterator<Item = Spc::Loc>;
    fn generation(&self) -> Gen;
    fn state(&self, location: &Spc::Loc) -> Option<S>;
}

pub trait Space<S: State<Gen>, Gen: Generation> {
    type Reg: Region<Self, S, Gen>;
    type Loc: Location<Self, S, Gen>;

    fn regions(&self, generation: &Gen) -> impl IntoIterator<Item = Self::Reg>;

    fn update_all(&mut self, generation: &Gen) -> Result<()>;

    fn reduce<A, F>(&self, generation: &Gen, init: A, f: F) -> A
    where
        F: Fn(&Self::Reg, &Self::Loc, A) -> A,
    {
        let mut accumulator = init;
        for region in self.regions(generation) {
            for location in region.locations() {
                accumulator = f(&region, &location, accumulator);
            }
        }
        accumulator
    }
}

pub trait Location<Spc: Space<S, Gen> + ?Sized, S: State<Gen>, Gen: Generation>: Sized {
    fn effectors(&self, space: &Spc) -> Result<impl IntoIterator<Item = Self>>;
    fn id(&self) -> String;
}

pub trait State<Gen: Generation>: Debug + Clone + Display {
    fn update<Spc: Space<Self, Gen>>(
        space: &Spc,
        region: &Spc::Reg,
        location: &Spc::Loc,
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
