// A5 (iii) companion: the hazard. A reference into a pool kept across an insertion.
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
pub fn misuse(reg: &mut Registry) {
    let p = reg.get_mut::<i32>(0);
    reg.emplace(5i32);
    *p += 1;
}
