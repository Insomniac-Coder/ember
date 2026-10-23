// A13 workaround: the trait requires Any (trait upcasting, Rust 1.86+), T: 'static everywhere.
use std::any::{Any, TypeId};
use std::collections::HashMap;

pub trait AnyPool: Any { fn len(&self) -> usize; }
pub struct Pool<T> { pub dense: Vec<T> }
impl<T: 'static> AnyPool for Pool<T> { fn len(&self) -> usize { self.dense.len() } }

pub struct Registry { pools: HashMap<TypeId, Box<dyn AnyPool>> }
impl Registry {
    pub fn get_mut<T: 'static>(&mut self, i: usize) -> &mut T {
        let pool: &mut dyn Any = self.pools.get_mut(&TypeId::of::<T>()).unwrap().as_mut();
        let pool = pool.downcast_mut::<Pool<T>>().unwrap();
        &mut pool.dense[i]
    }
}
