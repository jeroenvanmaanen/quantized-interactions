use std::{collections::HashMap, iter::repeat};

use log::info;

use crate::{
    patch::{
        AtMostSixEffectors, Effectors, SMALL_PATCH_SIZE, SmallIndexType,
        torus::{PatchLinks, TorusPatchLinks, calculate_grid, prepare_shuffle},
    },
    structure::{Generation, State},
};

use super::PatchTorus;

pub fn info_hexagons<S, Gen>(torus: &PatchTorus<S, Gen, TorusPatchLinks>)
where
    S: State<Gen> + Copy,
    Gen: Generation,
{
    info!("# Crystal hexagons info");
    info!("");
    let crystal = &torus.crystal;
    let width = torus.dimensions[0];
    let height = torus.dimensions[1];
    let (w, h) = calculate_grid(width, height);
    let wide = w > 1;
    let tall = h > 1;
    for i in 0..crystal.patch_count() {
        let patch_links = &crystal.patch_links[i];
        info!("## Patch: {i}: edges");
        let projections = |e: &HashMap<SmallIndexType, (usize, SmallIndexType)>, x, y, w| {
            vec![
                index(x, y, w),
                e.get(&index(x, y, w))
                    .map(|v| v.0)
                    .unwrap_or(SMALL_PATCH_SIZE as usize) as SmallIndexType,
                e.get(&index(x, y, w))
                    .map(|v| v.1)
                    .unwrap_or(SMALL_PATCH_SIZE),
            ]
        };
        info_hexagon(
            patch_links.edges(),
            3,
            &projections,
            patch_links.total_width,
            patch_links.total_height,
            patch_links.even,
        );
        info!("");
        info!("## Patch: {i}: effectors");
        let shuffle = prepare_shuffle(
            patch_links.total_width,
            patch_links.total_height,
            wide,
            tall,
        );
        let projections = |e: &AtMostSixEffectors, x, y, w| {
            let index = shuffle(index(x, y, w));
            let mut result = vec![index];
            for effector in e.iter(index) {
                result.push(effector);
            }
            result
        };
        info_hexagon(
            patch_links.effectors(),
            7,
            &projections,
            patch_links.total_width,
            patch_links.total_height,
            patch_links.even,
        );
        info!("");
    }
}

fn info_hexagon<C>(
    context: &C,
    projection_count: u8,
    projections: &impl Fn(&C, SmallIndexType, SmallIndexType, SmallIndexType) -> Vec<SmallIndexType>,
    width: SmallIndexType,
    height: SmallIndexType,
    even: bool,
) {
    let mut indent = even;
    let mut header = "        ".to_owned();
    for x in 0..width {
        let xx = format!("  {x:02x}");
        header.push_str(&xx);
    }
    let ko = SMALL_PATCH_SIZE;
    info!("{}", header);
    info!("");
    for y in 0..height {
        let mut lines = Vec::new();
        for p in 0..projection_count {
            let mut line = format!("{y:02x}[{p:02x}]:");
            if indent {
                line.push_str("  ");
            }
            lines.push(line);
        }
        for x in 0..width {
            let values = projections(context, x, y, width);
            for (v, line) in values.iter().chain(repeat(&ko)).zip(lines.iter_mut()) {
                let vv = if *v == ko {
                    "  ??".to_owned()
                } else {
                    format!("  {v:02x}")
                };
                line.push_str(&vv);
            }
        }
        for line in lines {
            info!("{}", line);
        }
        indent = !indent;
    }
}

fn index(x: SmallIndexType, y: SmallIndexType, w: SmallIndexType) -> SmallIndexType {
    y * w + x
}
