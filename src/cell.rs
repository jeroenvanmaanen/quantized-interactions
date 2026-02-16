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
pub trait Region<S: State> {
    fn state(&self, location: &S::Loc, generation: &S::Gen) -> Option<S>;
}
pub trait Location<S: State>: Sized {
    fn neighbors(&self) -> Result<impl IntoIterator<Item = Self>>;
    fn id(&self) -> String;
}
pub trait State: Debug + Clone + Display {
    type Gen: Generation;
    type Reg: Region<Self>;
    type Loc: Location<Self>;

    fn update(region: &Self::Reg, location: &Self::Loc, generation: &Self::Gen) -> Result<Self>;
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

impl<S: State<Loc = Cell<S>, Reg = CellRegion>> Region<S> for CellRegion {
    fn state(&self, location: &<S as State>::Loc, generation: &<S as State>::Gen) -> Option<S> {
        location.state(self, generation)
    }
}

#[derive(Debug)]
pub struct Cell<S: State>(Rc<InnerCell<S>>);

impl<S: State> Clone for Cell<S> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<S: State> Hash for Cell<S> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.id.hash(state);
    }
}

impl<S: State> PartialEq for Cell<S> {
    fn eq(&self, other: &Self) -> bool {
        self.0.id == other.0.id
    }
}
impl<S: State> Eq for Cell<S> {}

impl<S: State> Location<S> for Cell<S> {
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

impl<S: State<Loc = Cell<S>, Reg = CellRegion>> Cell<S> {
    pub fn new(generation: S::Gen, state: S) -> Self {
        Cell(Rc::new(InnerCell::new(generation, state)))
    }

    pub fn has_state(&self, generation: &S::Gen) -> bool {
        let guard = self.0.state_map.read().ok();
        guard.map(|m| m.get(generation).is_some()).unwrap_or(false)
    }

    pub fn join(&self, other: &Self) -> Result<()> {
        connect_cells(self, other)?;
        connect_cells(other, self)?;
        trace!("Joined: [{:?}] <=> [{:?}]", self.0.id, other.0.id);
        Ok(())
    }

    fn state(&self, _region: &S::Reg, generation: &S::Gen) -> Option<S> {
        let guard = self.0.state_map.read().ok();
        guard.map(|m| m.get(generation).map(Clone::clone)).flatten()
    }

    pub fn update(&self, generation: &S::Gen) -> Result<()> {
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

fn connect_cells<S: State<Loc = Cell<S>>>(this: &Cell<S>, that: &Cell<S>) -> Result<()> {
    let mut neighbors_lock = this
        .0
        .neighbors
        .write()
        .map_err(|e| anyhow!("Could not get write lock: {e}"))?;
    neighbors_lock.insert(that.clone());
    trace!("Connected {} => {}", this.id(), that.id());
    Ok(())
}

struct InnerCell<S: State> {
    id: Uuid,
    state_map: RwLock<HashMap<S::Gen, S>>,
    neighbors: RwLock<HashSet<Cell<S>>>,
}

impl<S: State> InnerCell<S> {
    pub fn new(generation: S::Gen, state: S) -> Self {
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

impl<S: State> Debug for InnerCell<S> {
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
