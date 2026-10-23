// A5 (iii): the ECS case of ODR-286. Rust accepts a reference into a pool found by a map lookup.
use std::any::{Any, TypeId};
use std::collections::HashMap;

pub struct Pool<T> { pub dense: Vec<T> }
pub struct Registry { pools: HashMap<TypeId, Box<dyn Any>> }

impl Registry {
    pub fn get_mut<T: 'static>(&mut self, i: usize) -> &mut T {
        let pool = self.pools.get_mut(&TypeId::of::<T>()).unwrap()
            .downcast_mut::<Pool<T>>().unwrap();
        &mut pool.dense[i]
    }
    pub fn emplace<T: 'static>(&mut self, v: T) {
        self.pools.entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(Pool::<T> { dense: Vec::new() }))
            .downcast_mut::<Pool<T>>().unwrap().dense.push(v);
    }
}
pub fn use_it(reg: &mut Registry) {
    let p = reg.get_mut::<i32>(0);
    *p += 1;
    reg.emplace(5i32);      // fine: `p` is no longer used
}
