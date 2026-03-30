use anyhow::{Result, anyhow};
use image::{GrayImage, Luma};
use log::{debug, info};
use std::{
    fs::{OpenOptions, create_dir_all},
    path::PathBuf,
};

use crate::{
    structure::{Generation, GrayScale, Location, Space, State},
    torus::{GrayScaleTorus, Tiling, Torus},
};

impl<T: Torus<S, Gen>, Spc, S, L, Gen> GrayScaleTorus<Spc, S, Gen> for T
where
    Spc: Space<S, Gen, Loc = L>,
    S: State<Gen> + GrayScale + Copy,
    L: Location<Spc, S, Gen>,
    Gen: Generation,
{
    fn export(
        &self,
        _region: &Spc::Reg,
        generation: &Gen,
        context: &<S as GrayScale>::Context,
        export_dir: Option<&PathBuf>,
    ) -> Result<()> {
        if let Some(dir) = export_dir {
            create_dir_all(&dir)?;
            match self.tiling() {
                Tiling::Hexagons => export::<Self, S, Gen>(self, generation, context, &dir)?,
                _ => todo!(),
            }
        }
        Ok(())
    }
}

fn export<T, S, Gen>(
    torus: &T,
    generation: &Gen,
    context: &<S as GrayScale>::Context,
    dir: &PathBuf,
) -> Result<()>
where
    T: Torus<S, Gen>,
    S: State<Gen> + GrayScale + Copy,
    Gen: Generation,
{
    info!("Exporting generation [{generation:?}]");
    let dimensions = torus.dimensions();
    if dimensions.len() != 2 {
        return Err(anyhow!("Torus should be two-dimensional"));
    }
    let width = dimensions[0];
    let height = dimensions[1];
    let mut img = GrayImage::new((width * 4 + 2) as u32, (height * 3 + 1) as u32);

    let space = torus.space();
    for region in space.regions(generation) {
        info!("Exporting region [{region:?}]");
        for loc in space.locations(&region) {
            let (x, y) = torus.coordinates(&region, &loc);
            let xs = if (y % 2) == 0 { 2 } else { 0 };
            let gray = space
                .state(&region, &loc)
                .map(|s| s.gray_value(context))
                .unwrap_or(128);
            debug!(
                "Coordinates: ({x}, {y}) -> [{:?}]",
                space.state(&region, &loc)
            );
            let luma = [gray];
            let xo = (xs + 4 * x) as u32;
            let yo = 3 * y as u32;
            for xp in [1, 2] {
                for yp in 0..=3 {
                    img.put_pixel(xo + xp, yo + yp, Luma::from(luma.clone()));
                }
            }
            for xp in [0, 3] {
                for yp in [1, 2] {
                    img.put_pixel(xo + xp, yo + yp, Luma::from(luma.clone()));
                }
            }
        }
    }

    let mut file_path = dir.clone();
    file_path.push(format!("gen-{generation:?}.png"));
    let mut writer = OpenOptions::new()
        .create(true)
        .write(true)
        .open(file_path)?;
    img.write_to(&mut writer, image::ImageFormat::Png)?;
    Ok(())
}
