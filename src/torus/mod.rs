pub mod grayscale;
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
    type Spc: Space<S, Gen>;
    fn space(&self) -> &Self::Spc;
    fn info(&self, generation: &Gen);
    fn update_all_cells(&mut self, generation: &Gen) -> Result<()>;
    fn tiling(&self) -> Tiling;
    fn dimensions(&self) -> Vec<usize>;
    fn coordinates(
        &self,
        region: &<Self::Spc as Space<S, Gen>>::Reg,
        location: &<Self::Spc as Space<S, Gen>>::Loc,
    ) -> (usize, usize);
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
