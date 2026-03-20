//! # Patches of cells
//!
//! A patch is a collection of nearby cells. The idea is that the interior of a patch is substantially larger than its edges.
//! Each cell is affected by a number of effectors, *i.e.*, its current state is completely determined by its state in the previous generation and the state of its effectors in the previous generation.
//!
//! The structure of connections between effectors and affected cells can be more or less flexible.
//! One possibility is that all patches have the same structure, so there is a single `Effectors` struct for the entire space. This is modeled in struct `Crystal`.
//! On the other end of the spectrum each patch could have its own `Effectors` struct. This is not implemented yet.
//!
//! The effectors of a cell with index *i* in patch *p<sub>a</sub>* can be found by calling `iter` on the `Effectors` instance that governs patch *p<sub>a</sub>*.
//! Each invocation of `next` on the resulting iterator yields an index *e* that can be used to find the state of the effector.
//! The global index of the patch that contains the effector *p<sub>e</sub>* can be looked up in the `cell_patches` array in patch *p<sub>a</sub>*.
//! If cell_patches[e] equals -1, then the effector belongs to the interior of the same patch: *p<sub>e</sub>* equals *p<sub>a</sub>*; otherwise this effector belongs to the edge of this patch and to the interior of another patch.
//! The state of an effector on the edge can be looked up in the `cells` array in *p<sub>e</sub>* using the index found in the `cell_index` array in patch *p<sub>a</sub>*.

#![allow(dead_code)]

mod poc;
mod torus;

pub use poc::example as poc_example;
pub use torus::new_hexagonal;

use anyhow::{Result, anyhow};
use std::{collections::HashMap, marker::PhantomData};

use crate::structure::{Generation, State};

const PATCH_SIZE: u8 = 0xFF;
const INTERNAL: u8 = 0xD * 0xD;

pub struct Crystal<S: State<Gen> + Copy, Gen: Generation, E: Effectors> {
    adjacent: Vec<HashMap<u8, usize>>,
    effectors: Vec<E>,
    generations: HashMap<Gen, Vec<Patch<S, Gen>>>,
}

impl<S: State<Gen> + Copy, Gen: Generation, E: Effectors> Crystal<S, Gen, E> {
    pub fn new(
        patch_count: usize,
        generation: &Gen,
        init: S,
        effector_factory: impl Fn() -> E,
    ) -> Self {
        let mut patches = Vec::new();
        let mut adjacent = Vec::new();
        let mut effectors = Vec::new();
        for _ in 0..patch_count {
            patches.push(Patch::new_init(init));
            adjacent.push(HashMap::new());
            effectors.push(effector_factory())
        }
        let mut generations = HashMap::new();
        generations.insert(generation.clone(), patches);
        Crystal {
            adjacent,
            effectors,
            generations,
        }
    }

    pub fn join_patches(&mut self, this_index: usize, that_index: usize) -> Result<()> {
        if this_index > self.adjacent.len() || that_index > self.adjacent.len() {
            return Err(anyhow!(
                "Index out of bounds: [{:?}]: [{this_index:?}]: [{that_index:?}]",
                self.adjacent.len()
            ));
        }
        {
            let this_adjacent = &mut self.adjacent[this_index];
            this_adjacent.insert(next_adjacent(&this_adjacent)?, that_index);
        }
        {
            let that_adjacent = &mut self.adjacent[that_index];
            that_adjacent.insert(next_adjacent(&that_adjacent)?, this_index);
        }
        Ok(())
    }

    pub fn patch_count(&self) -> usize {
        self.adjacent.len()
    }
}

fn next_adjacent(map: &HashMap<u8, usize>) -> Result<u8> {
    let mo = map.keys().max();
    if let Some(m) = mo {
        if *m >= 0xFF {
            Err(anyhow!("Too many adjacent patches: [{m:?}]"))
        } else {
            Ok(m + 1)
        }
    } else {
        Ok(0)
    }
}

pub struct Patch<S: State<Gen> + Copy, Gen: Generation> {
    cells: [S; PATCH_SIZE as usize],
    cell_patch: [u8; PATCH_SIZE as usize],
    cell_index: [u8; PATCH_SIZE as usize],
    size: u8,
    _phantom: PhantomData<Gen>,
}

impl<S, Gen> Patch<S, Gen>
where
    S: State<Gen> + Copy,
    Gen: Generation,
{
    pub fn new_init(init: S) -> Self {
        Patch {
            cells: [init; PATCH_SIZE as usize],
            cell_patch: [0xFF; PATCH_SIZE as usize],
            cell_index: [0; PATCH_SIZE as usize],
            size: 0,
            _phantom: PhantomData,
        }
    }
}

pub struct LocationInCrystal<Gen: Generation> {
    patch: usize,
    index: u8,
    _phantom: PhantomData<Gen>,
}

pub struct EffectorIterator<'a> {
    effectors: &'a [u8],
    pos: usize,
    to_go: u8,
}

impl<'a> Iterator for EffectorIterator<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.to_go < 1 || self.effectors[self.pos] == 0xFF {
            None
        } else {
            let pos = self.pos;
            self.pos += 1;
            self.to_go -= 1;
            Some(self.effectors[pos])
        }
    }
}

pub trait Effectors: Default {
    fn iter<'a>(&'a self, index: u8) -> EffectorIterator<'a>;
    fn add(&mut self, index: u8, effector_index: u8) -> Result<u8>;
}

#[derive(Clone)]
pub struct AtMostSixEffectors {
    effector_counts: [u8; PATCH_SIZE as usize],
    effectors: [u8; 6 * PATCH_SIZE as usize],
}

impl Default for AtMostSixEffectors {
    fn default() -> Self {
        let mut result = Self {
            effector_counts: [0; PATCH_SIZE as usize],
            effectors: [0xFFu8; 6 * PATCH_SIZE as usize],
        };
        for i in 0..6 {
            result.effectors[i] = 0;
        }
        result
    }
}

impl Effectors for AtMostSixEffectors {
    fn iter<'a>(&'a self, index: u8) -> EffectorIterator<'a> {
        EffectorIterator {
            effectors: &self.effectors,
            pos: 6 * index as usize,
            to_go: 6,
        }
    }

    fn add(&mut self, index: u8, effector_index: u8) -> Result<u8> {
        let i = index as usize;
        let n = self.effector_counts[i] as usize;
        let base = 6 * i;
        for k in base..(base + n) {
            if self.effectors[k] == effector_index {
                return Ok(self.effector_counts[i]);
            }
        }
        if n >= 6 {
            return Err(anyhow!("Cannot add more than 6 effectors"));
        }
        self.effectors[6 * i + n] = effector_index;
        self.effector_counts[i] += 1;
        Ok(self.effector_counts[i])
    }
}
