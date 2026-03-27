use std::{collections::HashMap, iter::repeat};

use log::info;

use crate::{
    patch::{AtMostSixEffectors, Effectors},
    structure::{Generation, State},
};

use super::PatchTorus;

pub fn info_hexagons<S, Gen>(torus: &PatchTorus<S, Gen, AtMostSixEffectors>)
where
    S: State<Gen> + Copy,
    Gen: Generation,
{
    info!("# Crystal hexagons info");
    info!("");
    let crystal = &torus.crystal;
    for i in 0..crystal.patch_count() {
        let patch_links = &crystal.patch_links[i];
        info!("## Patch: {i}: edges");
        let projections = |e: &HashMap<u8, (usize, u8)>, x, y, w| {
            vec![
                index(x, y, w),
                e.get(&index(x, y, w)).map(|v| v.0).unwrap_or(0xFF) as u8,
                e.get(&index(x, y, w)).map(|v| v.1).unwrap_or(0xFF),
            ]
        };
        info_hexagon(
            &patch_links.edges,
            3,
            &projections,
            patch_links.width,
            patch_links.height,
            patch_links.even,
        );
        info!("");
        info!("## Patch: {i}: effectors");
        let projections = |e: &AtMostSixEffectors, x, y, w| {
            let index = index(x, y, w);
            let mut result = vec![index];
            for effector in e.iter(index) {
                result.push(effector);
            }
            result
        };
        info_hexagon(
            &patch_links.effectors,
            7,
            &projections,
            patch_links.width,
            patch_links.height,
            patch_links.even,
        );
        info!("");
    }
}

fn info_hexagon<C>(
    context: &C,
    projection_count: u8,
    projections: &impl Fn(&C, u8, u8, u8) -> Vec<u8>,
    width: u8,
    height: u8,
    even: bool,
) {
    let mut indent = even;
    let mut header = "        ".to_owned();
    for x in 0..width {
        let xx = format!("  {x:02x}");
        header.push_str(&xx);
    }
    let ko = 0xFFu8;
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

fn index(x: u8, y: u8, w: u8) -> u8 {
    y * w + x
}
