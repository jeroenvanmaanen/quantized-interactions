pub mod utils;

use std::path::PathBuf;

pub use utils::get_index;

use crate::structure::{Generation, GrayScale, Space, State};
use anyhow::Result;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Tiling {
    Orthogonal,
    OrthogonalAndDiagonal,
    AdjacentTriangles,
    TouchingTriangles,
    Hexagons,
}

pub trait Torus<S: State<Gen>, Gen: Generation>: Sized {
    fn update_all(&self, generation: &Gen) -> Result<()>;
    fn info(&self, generation: &Gen);
}

pub trait GrayScaleTorus<Spc, S, Gen>: Torus<S, Gen>
where
    Spc: Space<S, Gen>,
    S: State<Gen> + GrayScale,
    Gen: Generation,
{
    fn export(
        &self,
        _region: &Spc::Reg,
        generation: &Gen,
        context: &<S as GrayScale>::Context,
        export_dir: Option<&PathBuf>,
    ) -> Result<()>;
}
