use log::info;

use crate::{
    patch::Effectors,
    structure::{Generation, State},
};

use super::PatchTorus;

pub fn info_hexagons<S, Gen, E>(_torus: &PatchTorus<S, Gen, E>)
where
    S: State<Gen> + Copy,
    Gen: Generation,
    E: Effectors,
{
    info!("TODO");
}
