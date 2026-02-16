use anyhow::{Result, anyhow};
// use log::debug;
use log::trace;
use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    hash::Hash,
    rc::Rc,
    sync::RwLock,
};
use uuid::Uuid;

pub trait Generation: Hash + Eq + PartialEq + Debug + Clone {
    fn successor(&self) -> Self;
}
pub trait Region<S: State<Gen>, Gen: Generation> {
    type Loc: Location<S, Gen>;

    fn locations(&self) -> impl IntoIterator<Item = Self::Loc>;
    fn state(&self, location: &Self::Loc, generation: &Gen) -> Option<S>;
}
pub trait Space<S: State<Gen>, Gen: Generation> {
    type Reg: Region<S, Gen>;

    fn regions(&self) -> impl IntoIterator<Item = Self::Reg>;

    fn reduce<A, F>(&self, init: A, f: F) -> A
    where
        F: Fn(&Self::Reg, &<Self::Reg as Region<S, Gen>>::Loc, A) -> A,
    {
        let mut accumulator = init;
        for region in self.regions() {
            for location in region.locations() {
                accumulator = f(&region, &location, accumulator);
            }
        }
        accumulator
    }
}
pub trait Location<S: State<Gen>, Gen: Generation>: Sized {
    fn neighbors(&self) -> Result<impl IntoIterator<Item = Self>>;
    fn id(&self) -> String;
}
pub trait State<Gen: Generation>: Debug + Clone + Display {
    fn update<Reg: Region<Self, Gen>>(
        region: &Reg,
        location: &<Reg as Region<Self, Gen>>::Loc,
        generation: &Gen,
    ) -> Result<Self>;
}
pub trait GrayScale {
    type Context;
    fn gray_value(&self, context: &Self::Context) -> u8;
}

impl Generation for usize {
    fn successor(&self) -> Self {
        self + 1
    }
}

#[derive(Default)]
pub struct CellRegion;

impl<S: State<Gen>, Gen: Generation> Region<S, Gen> for CellRegion {
    type Loc = Cell<S, Gen>;

    fn locations(&self) -> impl IntoIterator<Item = Self::Loc> {
        HashSet::new()
    }

    fn state(&self, location: &Self::Loc, generation: &Gen) -> Option<S> {
        location.state(self, generation)
    }
}

#[derive(Debug)]
pub struct Cell<S: State<Gen>, Gen: Generation>(Rc<InnerCell<S, Gen>>);

impl<S: State<Gen>, Gen: Generation> Clone for Cell<S, Gen> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<S: State<Gen>, Gen: Generation> Hash for Cell<S, Gen> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.id.hash(state);
    }
}

impl<S: State<Gen>, Gen: Generation> PartialEq for Cell<S, Gen> {
    fn eq(&self, other: &Self) -> bool {
        self.0.id == other.0.id
    }
}
impl<S: State<Gen>, Gen: Generation> Eq for Cell<S, Gen> {}

impl<S: State<Gen>, Gen: Generation> Location<S, Gen> for Cell<S, Gen> {
    fn neighbors(&self) -> Result<impl IntoIterator<Item = Self>> {
        self.0.neighbors.read().map(|s| s.clone()).map_err(|e| {
            anyhow!(
                "Could not get read lock for neighbors of: {:?}: {:?}",
                self.0.id,
                e
            )
        })
    }

    fn id(&self) -> String {
        self.0.id.to_string()
    }
}

impl<S: State<Gen>, Gen: Generation> Cell<S, Gen> {
    pub fn new(generation: Gen, state: S) -> Self {
        Cell(Rc::new(InnerCell::new(generation, state)))
    }

    pub fn has_state(&self, generation: &Gen) -> bool {
        let guard = self.0.state_map.read().ok();
        guard.map(|m| m.get(generation).is_some()).unwrap_or(false)
    }

    pub fn join(&self, other: &Self) -> Result<()> {
        connect_cells(self, other)?;
        connect_cells(other, self)?;
        trace!("Joined: [{:?}] <=> [{:?}]", self.0.id, other.0.id);
        Ok(())
    }

    pub fn state<Reg: Region<S, Gen>>(&self, _region: &Reg, generation: &Gen) -> Option<S> {
        let guard = self.0.state_map.read().ok();
        guard.map(|m| m.get(generation).map(Clone::clone)).flatten()
    }

    pub fn update(&self, generation: &Gen) -> Result<()> {
        let next_gen = generation.successor();
        if self.has_state(&next_gen) {
            return Ok(());
        }
        let region = CellRegion::default();
        let new_state = S::update(&region, self, &generation)?;
        let mut guard = self
            .0
            .state_map
            .write()
            .map_err(|e| anyhow!("Unable to obtain write lock for cell: {e:?}"))?;
        guard.insert(next_gen, new_state);
        Ok(())
    }
}

fn connect_cells<S, Gen>(this: &Cell<S, Gen>, that: &Cell<S, Gen>) -> Result<()>
where
    S: State<Gen>,
    Gen: Generation,
{
    let mut neighbors_lock = this
        .0
        .neighbors
        .write()
        .map_err(|e| anyhow!("Could not get write lock: {e}"))?;
    neighbors_lock.insert(that.clone());
    trace!("Connected {} => {}", this.id(), that.id());
    Ok(())
}

struct InnerCell<S: State<Gen>, Gen: Generation> {
    id: Uuid,
    state_map: RwLock<HashMap<Gen, S>>,
    neighbors: RwLock<HashSet<Cell<S, Gen>>>,
}

impl<S: State<Gen>, Gen: Generation> InnerCell<S, Gen> {
    pub fn new(generation: Gen, state: S) -> Self {
        let id = Uuid::new_v4();
        let mut state_map = HashMap::new();
        state_map.insert(generation, state);
        let state_map = RwLock::new(state_map);
        let neighbors = RwLock::new(HashSet::new());
        InnerCell {
            id,
            state_map,
            neighbors,
        }
    }

    fn id(&self) -> Uuid {
        self.id
    }
}

impl<S: State<Gen>, Gen: Generation> Debug for InnerCell<S, Gen> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let neighbors = &self
            .neighbors
            .read()
            .ok()
            .map(|n| {
                n.iter()
                    .map(|c| c.0.id().clone())
                    .collect::<HashSet<Uuid>>()
            })
            .unwrap_or_else(|| HashSet::<Uuid>::new());
        f.debug_struct("InnerCell")
            .field("id", &self.id)
            .field("state_map", &self.state_map.read().ok())
            .field("neighbors", neighbors)
            .finish()
    }
}
