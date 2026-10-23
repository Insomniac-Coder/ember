// A13: an ECS registry of type-erased pools, found by type.
use std::any::TypeId;
use std::collections::HashMap;

pub trait AnyPool { fn len(&self) -> usize; }
pub struct Pool<T> { pub dense: Vec<T> }
impl<T> AnyPool for Pool<T> { fn len(&self) -> usize { self.dense.len() } }

pub struct Registry { pools: HashMap<TypeId, Box<dyn AnyPool>> }
impl Registry {
    pub fn get_mut<T: 'static>(&mut self, i: usize) -> &mut T {
        let pool = self.pools.get_mut(&TypeId::of::<T>()).unwrap();
        let pool = pool.downcast_mut::<Pool<T>>().unwrap();
        &mut pool.dense[i]
    }
}

// Object safety: a generic method makes the trait unusable as `dyn`.
pub trait Visitor { fn visit(&mut self, n: usize); }
pub trait Inspect { fn inspect<V: Visitor>(&self, v: &mut V); }
pub fn all(xs: &[Box<dyn Inspect>]) -> usize { xs.len() }
