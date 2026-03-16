#![allow(dead_code)]

mod poc;

pub use poc::example as poc_example;

use anyhow::{Result, anyhow};
use std::{collections::HashMap, marker::PhantomData, rc::Rc};

use crate::structure::{Generation, State};

const PATCH_SIZE: u8 = 0xFF;
const INTERNAL: u8 = 0xD * 0xD;

pub struct Inflexible<S: State<Gen> + Copy, Gen: Generation, N: Neighbors> {
    adjacent: Vec<HashMap<u8, usize>>,
    generations: HashMap<Gen, Vec<Patch<S, Gen, N>>>,
}

impl<S: State<Gen> + Copy, Gen: Generation, N: Neighbors + Clone> Inflexible<S, Gen, N> {
    pub fn new(neighbors: &N, capacity: usize, generation: &Gen, init: S) -> Self {
        let patch_count = capacity / (INTERNAL as usize);
        let mut patches = Vec::new();
        let mut adjacent = Vec::new();
        for _ in 0..patch_count {
            patches.push(Patch::new_init(neighbors.clone(), init));
            adjacent.push(HashMap::new());
        }
        let mut generations = HashMap::new();
        generations.insert(generation.clone(), patches);
        Inflexible {
            adjacent,
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

pub struct Patch<S: State<Gen> + Copy, Gen: Generation, N: Neighbors> {
    cells: [S; PATCH_SIZE as usize],
    cell_patch: [u8; PATCH_SIZE as usize],
    neighbors: N,
    size: u8,
    _phantom: PhantomData<Gen>,
}

impl<S, Gen, N> Patch<S, Gen, N>
where
    S: State<Gen> + Copy,
    Gen: Generation,
    N: Neighbors,
{
    pub fn new_init(neighbors: N, init: S) -> Self {
        Patch {
            cells: [init; PATCH_SIZE as usize],
            cell_patch: [0xFF; PATCH_SIZE as usize],
            neighbors,
            size: 0,
            _phantom: PhantomData,
        }
    }
}

pub struct Location<S: State<Gen> + Copy, Gen: Generation, N: Neighbors> {
    patch: Rc<Patch<S, Gen, N>>,
    index: u8,
    _phantom: PhantomData<Gen>,
}

pub struct NeighborIterator<'a> {
    neighbors: &'a [u8],
    pos: usize,
    to_go: u8,
}

impl<'a> Iterator for NeighborIterator<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.to_go < 1 || self.neighbors[self.pos] == 0xFF {
            None
        } else {
            let pos = self.pos;
            self.pos += 1;
            self.to_go -= 1;
            Some(self.neighbors[pos])
        }
    }
}

pub trait Neighbors: Default {
    fn neighbors<'a>(&'a self, index: u8) -> NeighborIterator<'a>;
    fn add(&mut self, index: u8, neighbor_index: u8) -> Result<u8>;
}

#[derive(Clone)]
pub struct AtMostSixNeighbors {
    neighbor_counts: [u8; PATCH_SIZE as usize],
    neighbors: [u8; 6 * PATCH_SIZE as usize],
}

impl Default for AtMostSixNeighbors {
    fn default() -> Self {
        let mut result = Self {
            neighbor_counts: [0; PATCH_SIZE as usize],
            neighbors: [0xFFu8; 6 * PATCH_SIZE as usize],
        };
        for i in 0..6 {
            result.neighbors[i] = 0;
        }
        result
    }
}

impl Neighbors for AtMostSixNeighbors {
    fn neighbors<'a>(&'a self, index: u8) -> NeighborIterator<'a> {
        NeighborIterator {
            neighbors: &self.neighbors,
            pos: 6 * index as usize,
            to_go: 6,
        }
    }

    fn add(&mut self, index: u8, neighbor_index: u8) -> Result<u8> {
        let i = index as usize;
        let n = self.neighbor_counts[i] as usize;
        let base = 6 * i;
        for k in base..(base + n) {
            if self.neighbors[k] == neighbor_index {
                return Ok(self.neighbor_counts[i]);
            }
        }
        if n >= 6 {
            return Err(anyhow!("Cannot add more than 6 neighbors"));
        }
        self.neighbors[6 * i + n] = neighbor_index;
        self.neighbor_counts[i] += 1;
        Ok(self.neighbor_counts[i])
    }
}
