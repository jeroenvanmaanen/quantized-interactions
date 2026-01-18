mod cell;
mod conway;
mod experiment;
mod torus;

use anyhow::Result;
use log::info;

pub fn main() -> Result<()> {
    info!("Quantized interactions");

    conway::example()?;

    experiment::example()?;

    Ok(())
}
