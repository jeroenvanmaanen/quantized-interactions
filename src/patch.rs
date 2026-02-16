#![allow(dead_code)]

use anyhow::{Result, anyhow};
use std::{collections::HashMap, rc::Rc};

use crate::cell::State;

const PATCH_SIZE: u8 = 0xFF;

pub struct Inflexible<S: State + Copy, N: Neigbors> {
    adjacent: Vec<HashMap<u8, usize>>,
    generations: HashMap<S::Gen, Vec<Patch<S, N>>>,
}

pub struct Patch<S: State + Copy, N: Neigbors> {
    cells: [S; PATCH_SIZE as usize],
    cell_patch: [u8; PATCH_SIZE as usize],
    neighbors: N,
    size: u8,
}

impl<S, N: Neigbors> Patch<S, N>
where
    S: State + Default + Copy,
{
    pub fn new_init(neighbors: N, init: S) -> Self {
        Patch {
            cells: [init; PATCH_SIZE as usize],
            cell_patch: [0xFF; PATCH_SIZE as usize],
            neighbors,
            size: 0,
        }
    }
}

pub struct Location<S: State + Copy, N: Neigbors> {
    patch: Rc<Patch<S, N>>,
    index: u8,
}

pub struct NeighborIterator<'a> {
    neigbors: &'a [u8],
    pos: usize,
    to_go: u8,
}

impl<'a> Iterator for NeighborIterator<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.to_go < 1 || self.neigbors[self.pos] == 0xFF {
            None
        } else {
            let pos = self.pos;
            self.pos += 1;
            self.to_go -= 1;
            Some(self.neigbors[pos])
        }
    }
}

pub trait Neigbors: Default {
    fn neighbors<'a>(&'a self, index: u8) -> NeighborIterator<'a>;
    fn add(&mut self, index: u8, neighbor_index: u8) -> Result<u8>;
}

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

impl Neigbors for AtMostSixNeighbors {
    fn neighbors<'a>(&'a self, index: u8) -> NeighborIterator<'a> {
        NeighborIterator {
            neigbors: &self.neighbors,
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
