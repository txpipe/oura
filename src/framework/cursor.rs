use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

use pallas::network::miniprotocols::Point;

#[derive(Clone)]
pub enum Intersection {
    Tip,
    Origin,
    Breadcrumbs(VecDeque<Point>),
}

const HARDCODED_BREADCRUMBS: usize = 20;

// TODO: include exponential breadcrumbs logic here
#[derive(Clone)]
pub struct Cursor(Arc<RwLock<Intersection>>);

impl Cursor {
    pub fn new(value: Intersection) -> Self {
        Self(Arc::new(RwLock::new(value)))
    }

    pub fn read(&self) -> Intersection {
        let v = self.0.read().unwrap();
        v.clone()
    }

    pub fn latest_known_point(&self) -> Option<Point> {
        let guard = self.0.read().unwrap();

        match &*guard {
            Intersection::Breadcrumbs(v) => v.front().cloned(),
            _ => None,
        }
    }

    pub fn add_breadcrumb(&self, value: Point) {
        let mut guard = self.0.write().unwrap();

        match &mut *guard {
            Intersection::Tip | Intersection::Origin => {
                *guard = Intersection::Breadcrumbs(VecDeque::from(vec![value]));
            }
            Intersection::Breadcrumbs(crumbs) => {
                crumbs.push_front(value);

                if crumbs.len() > HARDCODED_BREADCRUMBS {
                    crumbs.pop_back();
                }
            }
        }
    }
}

impl From<Intersection> for pallas::upstream::Intersection {
    fn from(value: Intersection) -> Self {
        match value {
            Intersection::Tip => Self::Tip,
            Intersection::Origin => Self::Origin,
            Intersection::Breadcrumbs(x) => Self::Breadcrumbs(Vec::from(x)),
        }
    }
}

impl pallas::upstream::Cursor for Cursor {
    fn intersection(&self) -> pallas::upstream::Intersection {
        self.read().into()
    }
}
