pub mod utils;

use std::path::PathBuf;

pub use utils::get_index;

use crate::{
    cell::Cell,
    structure::{Generation, GrayScale, Region, State},
};
use anyhow::Result;

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum Tiling {
    Orthogonal,
    OrthogonalAndDiagonal,
    AdjacentTriangles,
    TouchingTriangles,
    Hexagons,
}

pub trait Torus<S: State<Gen>, Gen: Generation>: Sized {
    fn new<F>(
        tiling: Tiling,
        dimensions: &[usize],
        initial_gen: Gen,
        initial_state: F,
    ) -> Result<Self>
    where
        F: Fn(&[usize]) -> S;

    fn update_all(&self, generation: &Gen) -> Result<()>;

    fn info(&self, generation: &Gen);
}

pub trait GrayScaleTorus<S: State<Gen> + GrayScale, Gen: Generation>: Torus<S, Gen> {
    fn export<Reg>(
        &self,
        _region: &Reg,
        generation: &Gen,
        context: &<S as GrayScale>::Context,
        export_dir: Option<&PathBuf>,
    ) -> Result<()>
    where
        Reg: Region<S, Gen, Loc = Cell<S, Gen>>;
}
