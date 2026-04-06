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

use log::debug;
pub use poc::example as poc_example;
pub use torus::new_hexagonal_torus;

use anyhow::{Result, anyhow};
use std::{cell::RefCell, collections::HashMap, fmt::Debug, ops::Range, rc::Rc};

use crate::structure::{Generation, Location, Region, Space, State};

const PATCH_SIZE: u8 = 0xFF;
const INTERNAL: u8 = 0xD * 0xD;

pub struct Crystal<S: State<Gen> + Copy, Gen: Generation, PL: PatchLinks> {
    patch_links: Vec<PL>,
    generations: HashMap<Gen, Vec<Rc<RefCell<Patch<S, Gen>>>>>,
}

pub trait PatchLinks {
    type Eff: Effectors;

    fn effectors(&self) -> &Self::Eff;
    fn edges(&self) -> &HashMap<u8, (usize, u8)>;
}

impl<S, Gen, PL> Crystal<S, Gen, PL>
where
    S: State<Gen> + Copy,
    Gen: Generation,
    PL: PatchLinks<Eff: Effectors>,
{
    pub fn new(
        patch_count: usize,
        generation: &Gen,
        init: S,
        patch_links_factory: impl Fn() -> PL,
    ) -> Self {
        let mut patches = Vec::new();
        let mut patch_links = Vec::new();
        for index in 0..patch_count {
            let new_patch = Patch::new_init(init, index, generation.clone());
            patches.push(Rc::new(RefCell::new(new_patch)));
            patch_links.push(patch_links_factory());
        }
        let mut generations = HashMap::new();
        generations.insert(generation.clone(), patches);
        Crystal {
            patch_links,
            generations,
        }
    }

    pub fn patch_count(&self) -> usize {
        self.patch_links.len()
    }

    fn stitch(&self, patch: &mut Patch<S, Gen>, generation: &Gen) {
        if let Some(patches) = self.generations.get(generation) {
            let edges = &self.patch_links[patch.index].edges();
            for i in patch.size..patch.total_size {
                if let Some((other_index, j)) = edges.get(&i) {
                    let other = patches[*other_index].borrow();
                    let state = other.cells[*j as usize];
                    patch.cells[i as usize] = state;
                }
            }
        }
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

impl<S, Gen, PL> Space<S, Gen> for Crystal<S, Gen, PL>
where
    S: State<Gen> + Copy,
    Gen: Generation,
    PL: PatchLinks,
{
    type Reg = Rc<RefCell<Patch<S, Gen>>>;
    type Loc = LocationInPatch;

    fn regions(&self, generation: &Gen) -> impl IntoIterator<Item = Self::Reg> {
        self.generations
            .get(generation)
            .map(Clone::clone)
            .unwrap_or_else(|| Vec::new())
    }

    fn update_all(&mut self, generation: &Gen) -> Result<()> {
        let patches = &self.generations[generation];
        let next_generation = generation.successor();
        let mut updated_patches = Vec::new();
        debug!("Number of patches: [{generation:?}]: {}", patches.len());
        for patch_index in 0..patches.len() {
            let patch_ref = &patches[patch_index];
            let mut patch = patch_ref.borrow_mut();
            self.stitch(&mut patch, generation);
            drop(patch); // Drop temporary mutable borrow
            let patch = patch_ref.borrow();
            let mut updated_patch = patch.clone();
            updated_patch.generation = next_generation.clone();
            debug!("Patch size: {}", patch.size);
            for i in 0..patch.size {
                let location = LocationInPatch {
                    index: i,
                    patch: patch.index,
                };
                if self.patch_links[patch_index]
                    .effectors()
                    .iter(i)
                    .next()
                    .is_some()
                {
                    let new_state = S::update(self, patch_ref, &location)?;
                    updated_patch.cells[i as usize] = new_state;
                }
            }
            updated_patches.push(Rc::new(RefCell::new(updated_patch)));
        }
        self.generations.insert(next_generation, updated_patches);
        Ok(())
    }
}

#[derive(Clone)]
pub struct Patch<S: State<Gen> + Copy, Gen: Generation> {
    cells: [S; PATCH_SIZE as usize],
    cell_patch: [u8; PATCH_SIZE as usize],
    cell_index: [u8; PATCH_SIZE as usize],
    index: usize,
    generation: Gen,
    size: u8,
    total_size: u8, // Includes edges
}

impl<S, Gen> Patch<S, Gen>
where
    S: State<Gen> + Copy,
    Gen: Generation,
{
    pub fn new_init(init: S, index: usize, generation: Gen) -> Self {
        Patch {
            cells: [init; PATCH_SIZE as usize],
            cell_patch: [0xFF; PATCH_SIZE as usize],
            cell_index: [0; PATCH_SIZE as usize],
            index,
            generation,
            size: 0,
            total_size: 0,
        }
    }
}

impl<Spc, S, Gen> Region<Spc, S, Gen> for Rc<RefCell<Patch<S, Gen>>>
where
    Spc: Space<S, Gen, Loc = LocationInPatch>,
    S: State<Gen> + Copy,
    Gen: Generation,
{
    fn locations(&self) -> impl IntoIterator<Item = Spc::Loc> {
        let patch = self.borrow();
        AllLocationsInPatchIterator {
            inner: 0..patch.size,
            patch: patch.index,
        }
    }

    fn generation(&self) -> Gen {
        self.borrow().generation.clone()
    }

    fn state(&self, location: &Spc::Loc) -> Option<S> {
        let i = location.index;
        let region = self.borrow();
        if i < region.size {
            Some(region.cells[location.index as usize])
        } else {
            None
        }
    }
}

impl<S, Gen> Debug for Patch<S, Gen>
where
    S: State<Gen> + Copy,
    Gen: Generation,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Patch")
            .field("generation", &self.generation)
            .field("index", &self.index)
            .finish()
    }
}
pub struct LocationInPatch {
    patch: usize,
    index: u8,
}

impl<S, Gen, PL> Location<Crystal<S, Gen, PL>, S, Gen> for LocationInPatch
where
    S: State<Gen> + Copy,
    Gen: Generation,
    PL: PatchLinks,
{
    fn effectors(&self, space: &Crystal<S, Gen, PL>) -> Result<impl IntoIterator<Item = Self>> {
        let patch_effectors = &space.patch_links[self.patch].effectors();
        let cell_effectors = patch_effectors
            .iter(self.index)
            .map(|i| LocationInPatch {
                index: i,
                patch: self.patch,
            })
            .collect::<Vec<Self>>();
        // debug!(
        //     "Effectors: [{}]: [{}]: [{}]: {:?}.",
        //     self.patch,
        //     self.index,
        //     cell_effectors.len(),
        //     cell_effectors.iter().map(|l| l.index).collect::<Vec<u8>>()
        // );
        Ok(cell_effectors)
    }

    fn id(&self) -> String {
        todo!()
    }
}

pub struct AllLocationsInPatchIterator {
    inner: Range<u8>,
    patch: usize,
}

impl Iterator for AllLocationsInPatchIterator {
    type Item = LocationInPatch;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(i) = self.inner.next() {
            Some(LocationInPatch {
                index: i,
                patch: self.patch,
            })
        } else {
            None
        }
    }
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
    fn debug<S: AsRef<str>>(&self, label: S);
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
            to_go: self.effector_counts[index as usize],
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
        self.effectors[base + n] = effector_index;
        self.effector_counts[i] += 1;
        Ok(self.effector_counts[i])
    }

    fn debug<S: AsRef<str>>(&self, label: S) {
        for i in 0..self.effector_counts.len() {
            let count = self.effector_counts[i];
            if count > 0 {
                debug!("{}: {i}: {count}", label.as_ref());
            }
        }
    }
}
