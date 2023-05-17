use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

use pallas::network::miniprotocols::Point;

const HARDCODED_BREADCRUMBS: usize = 20;

type State = VecDeque<Point>;

// TODO: include exponential breadcrumbs logic here
#[derive(Clone)]
pub struct Cursor(Arc<RwLock<State>>);

impl Cursor {
    pub fn new(state: State) -> Self {
        Self(Arc::new(RwLock::new(state)))
    }

    pub fn is_empty(&self) -> bool {
        let v = self.0.read().unwrap();
        v.is_empty()
    }

    pub fn clone_state(&self) -> State {
        let v = self.0.read().unwrap();
        v.clone()
    }

    pub fn latest_known_point(&self) -> Option<Point> {
        let state = self.0.read().unwrap();
        state.front().cloned()
    }

    pub fn add_breadcrumb(&self, value: Point) {
        let mut state = self.0.write().unwrap();

        state.push_front(value);

        if state.len() > HARDCODED_BREADCRUMBS {
            state.pop_back();
        }
    }
}
