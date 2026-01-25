mod cell;
mod conway;
mod experiment;
mod torus;
mod wave;

use anyhow::Result;
use log::info;

pub fn main() -> Result<()> {
    info!("Quantized interactions");

    conway::example()?;

    experiment::example()?;

    wave::example()?;
    // wave::debug()?;

    Ok(())
}
