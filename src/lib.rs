mod cell;
mod conway;
mod torus;

use anyhow::Result;
use cell::Cell;
use conway::Conway;
use log::info;

use crate::{cell::Generation, torus::Torus};

pub fn main() -> Result<()> {
    info!("Quantized interactions");
    let origin = Cell::new(0u32, Conway::new(false));
    let other = Cell::new(1u32, Conway::new(true));
    origin.join(&other)?;
    info!("Origin: [{origin:?}]");
    let width = 5;
    let height = 5;
    let generation = 0u32;
    let torus = Torus::new(
        origin.clone(),
        torus::Tiling::TouchingSquares,
        width,
        height,
        generation.clone(),
        |x: usize, y: usize| Conway::new(y == 2 && (x >= 1 && x <= 3)),
    )?;
    info!("Origin: [{origin:?}]");
    torus.info(&generation);
    torus.update_all(&0u32)?;
    let generation = generation.successor();
    torus.info(&generation);
    Ok(())
}
