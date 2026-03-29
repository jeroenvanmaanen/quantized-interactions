use anyhow::{Result, anyhow};
use image::{GrayImage, Luma};
use std::{
    fs::{OpenOptions, create_dir_all},
    path::PathBuf,
};

use crate::{
    patch::{
        LocationInPatch,
        torus::{PatchTorus, TorusPatchLinks},
    },
    structure::{Generation, GrayScale, Space, State},
    torus::{GrayScaleTorus, Tiling},
};

impl<Spc, S, Gen> GrayScaleTorus<Spc, S, Gen> for PatchTorus<S, Gen, TorusPatchLinks>
where
    Spc: Space<S, Gen, Loc = LocationInPatch>,
    S: State<Gen> + GrayScale + Copy,
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
            match self.tiling {
                Tiling::Hexagons => export::<S, Gen>(self, generation, context, &dir)?,
                _ => todo!(),
            }
        }
        Ok(())
    }
}

fn export<S, Gen>(
    torus: &PatchTorus<S, Gen, TorusPatchLinks>,
    generation: &Gen,
    context: &<S as GrayScale>::Context,
    dir: &PathBuf,
) -> Result<()>
where
    S: State<Gen> + GrayScale + Copy,
    Gen: Generation,
{
    if torus.dimensions.len() != 2 {
        return Err(anyhow!("Torus should be two-dimensional"));
    }
    let width = torus.dimensions[0];
    let height = torus.dimensions[1];
    let mut img = GrayImage::new((width * 4 + 2) as u32, (height * 3 + 1) as u32);

    let crystal = &torus.crystal;
    for region in crystal.regions(generation) {
        for loc in crystal.locations(&region) {
            let (x, y) = torus.coordinates(region.clone(), &loc);
            let xs = if (y % 2) == 0 { 2 } else { 0 };
            let gray = (crystal.state(&region, &loc) as Option<S>)
                .map(|s| s.gray_value(context))
                .unwrap_or(128);
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
