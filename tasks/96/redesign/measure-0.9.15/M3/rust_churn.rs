// W3 "churn" in Rust: the same work as plain_churn_*.c (2^18 slots each owning one 32-byte object
// from one of two pool instances; 2^25 steps; an object whose hp runs out ends and a new one is made
// in a random team's pool, the new one made before the old one ends, as `=` does). One binary,
// three forms, chosen by the first argument:
//   box    idiomatic safe Rust with no pool: `Box<Obj>` from the global allocator; a slot 8 bytes.
//   pbox   the rule's twin: `PBox<'a, Obj>` = `Box<Obj, &'a Pool>` (block + pool address, 16
//          bytes), its drop giving the block back to its own pool through the kept `&Pool`. The
//          pool is hand-written (unsafe inside, as an allocator is), the same algorithm as m3_rt.h's
//          Pool: a free list, 64 KB slabs with a 64-byte header.
//   slab   safe Rust's index idiom (the `slab` crate, hand-written): two `Vec<Entry>` with a free
//          list of indices; a slot is a u32 (top bit: the team), every access bounds-checked and
//          matched. An entry is 40 bytes (a different layout: noted in M3.md).
#[path = "rharness.rs"]
mod harness;

use harness::Rng;
use std::alloc::{alloc, dealloc, Layout};
use std::cell::{Cell, RefCell};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

const S: usize = 1 << 18;
const STEPS: u32 = 1 << 25;

struct Obj {
    id: i64,
    x: i32,
    y: i32,
    hp: i32,
    team: i32,
    score: i64,
}
const _: () = assert!(std::mem::size_of::<Obj>() == 32);

#[inline(always)]
fn new_obj(id: i64, i: u32, r: u32) -> Obj {
    Obj { id, x: i as i32, y: ((r >> 8) & 1023) as i32, hp: (1 + (r >> 20) % 16) as i32, team: (r & 1) as i32, score: 0 }
}

// ---- the pool (pbox)
const SLAB: usize = 1 << 16;
const HEADER: usize = 64;
struct Pool {
    free: Cell<*mut u8>,
    cur: Cell<usize>,
    end: Cell<usize>,
    bsize: usize,
    slabs: RefCell<Vec<*mut u8>>,
}
impl Pool {
    fn new(bsize: usize) -> Pool {
        Pool { free: Cell::new(std::ptr::null_mut()), cur: Cell::new(0), end: Cell::new(0), bsize, slabs: RefCell::new(Vec::new()) }
    }
    #[inline(always)]
    fn alloc(&self) -> *mut u8 {
        let b = self.free.get();
        if !b.is_null() {
            self.free.set(unsafe { *(b as *mut *mut u8) });
            return b;
        }
        if self.end.get() - self.cur.get() >= self.bsize {
            let b = self.cur.get();
            self.cur.set(b + self.bsize);
            return b as *mut u8;
        }
        self.slow()
    }
    #[inline(never)]
    #[cold]
    fn slow(&self) -> *mut u8 {
        let s = unsafe { alloc(Layout::from_size_align(SLAB, SLAB).unwrap()) };
        if s.is_null() {
            return s;
        }
        self.slabs.borrow_mut().push(s);
        let start = s as usize + HEADER;
        self.cur.set(start + self.bsize);
        self.end.set(s as usize + SLAB);
        start as *mut u8
    }
    #[inline(always)]
    fn free(&self, b: *mut u8) {
        unsafe { *(b as *mut *mut u8) = self.free.get() };
        self.free.set(b);
    }
}
impl Drop for Pool {
    fn drop(&mut self) {
        for &s in self.slabs.borrow().iter() {
            unsafe { dealloc(s, Layout::from_size_align(SLAB, SLAB).unwrap()) };
        }
    }
}

/// `Box<T, &'a Pool>`.
struct PBox<'a, T> {
    p: NonNull<T>,
    a: &'a Pool,
}
impl<'a, T> PBox<'a, T> {
    #[inline(always)]
    fn new_in(v: T, a: &'a Pool) -> PBox<'a, T> {
        let p = match NonNull::new(a.alloc() as *mut T) {
            Some(p) => p,
            None => std::alloc::handle_alloc_error(Layout::new::<T>()),
        };
        unsafe { p.as_ptr().write(v) };
        PBox { p, a }
    }
}
impl<'a, T> Deref for PBox<'a, T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        unsafe { self.p.as_ref() }
    }
}
impl<'a, T> DerefMut for PBox<'a, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.p.as_mut() }
    }
}
impl<'a, T> Drop for PBox<'a, T> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { std::ptr::drop_in_place(self.p.as_ptr()) };
        self.a.free(self.p.as_ptr() as *mut u8);
    }
}
const _: () = assert!(std::mem::size_of::<PBox<Obj>>() == 16);

// ---- the slab (safe)
enum Entry {
    Occupied(Obj),
    Vacant(u32),
}
const NONE: u32 = u32::MAX;
struct Slab {
    entries: Vec<Entry>,
    free: u32,
}
impl Slab {
    fn new() -> Slab {
        Slab { entries: Vec::new(), free: NONE }
    }
    #[inline(always)]
    fn insert(&mut self, o: Obj) -> u32 {
        if self.free != NONE {
            let i = self.free;
            self.free = match self.entries[i as usize] {
                Entry::Vacant(next) => next,
                Entry::Occupied(_) => unreachable!(),
            };
            self.entries[i as usize] = Entry::Occupied(o);
            i
        } else {
            self.entries.push(Entry::Occupied(o));
            (self.entries.len() - 1) as u32
        }
    }
    #[inline(always)]
    fn remove(&mut self, i: u32) {
        self.entries[i as usize] = Entry::Vacant(self.free);
        self.free = i;
    }
    #[inline(always)]
    fn get_mut(&mut self, i: u32) -> &mut Obj {
        match &mut self.entries[i as usize] {
            Entry::Occupied(o) => o,
            Entry::Vacant(_) => panic!("a vacant entry"),
        }
    }
}
const TEAM_BIT: u32 = 1 << 31;

fn main() {
    let form = std::env::args().nth(1).expect("form: box | pbox | slab");
    let t0 = harness::start();
    let mut rng = Rng::new(42);
    let mut cs: u64 = 0;
    let mut next_id: i64 = 0;
    match form.as_str() {
        "box" => {
            let mut slots: Vec<Box<Obj>> = Vec::with_capacity(S);
            for i in 0..S as u32 {
                let r = rng.next();
                slots.push(Box::new(new_obj(next_id, i, r)));
                next_id += 1;
            }
            for _ in 0..STEPS {
                let r = rng.next();
                let i = ((r >> 3) as usize) % S;
                let o = &mut slots[i];
                o.hp -= ((r >> 25) & 7) as i32;
                o.score += (o.x + o.y) as i64;
                cs = cs.wrapping_add((o.score ^ o.id) as u64);
                if o.hp <= 0 {
                    cs = cs.wrapping_add(o.score as u64);
                    let r2 = rng.next();
                    slots[i] = Box::new(new_obj(next_id, i as u32, r2));
                    next_id += 1;
                }
            }
            drop(slots);
        }
        "pbox" => {
            let pools = [Pool::new(32), Pool::new(32)];
            {
                let mut slots: Vec<PBox<Obj>> = Vec::with_capacity(S);
                for i in 0..S as u32 {
                    let r = rng.next();
                    slots.push(PBox::new_in(new_obj(next_id, i, r), &pools[(r & 1) as usize]));
                    next_id += 1;
                }
                for _ in 0..STEPS {
                    let r = rng.next();
                    let i = ((r >> 3) as usize) % S;
                    let o = &mut *slots[i];
                    o.hp -= ((r >> 25) & 7) as i32;
                    o.score += (o.x + o.y) as i64;
                    cs = cs.wrapping_add((o.score ^ o.id) as u64);
                    if o.hp <= 0 {
                        cs = cs.wrapping_add(o.score as u64);
                        let r2 = rng.next();
                        slots[i] = PBox::new_in(new_obj(next_id, i as u32, r2), &pools[(r2 & 1) as usize]);
                        next_id += 1;
                    }
                }
            }
        }
        "slab" => {
            let mut slabs = [Slab::new(), Slab::new()];
            let mut slots: Vec<u32> = Vec::with_capacity(S);
            let place = |slabs: &mut [Slab; 2], o: Obj| -> u32 {
                let team = o.team as usize;
                let k = slabs[team].insert(o);
                if team == 1 { k | TEAM_BIT } else { k }
            };
            for i in 0..S as u32 {
                let r = rng.next();
                slots.push(place(&mut slabs, new_obj(next_id, i, r)));
                next_id += 1;
            }
            for _ in 0..STEPS {
                let r = rng.next();
                let i = ((r >> 3) as usize) % S;
                let h = slots[i];
                let team = (h >> 31) as usize;
                let o = slabs[team].get_mut(h & !TEAM_BIT);
                o.hp -= ((r >> 25) & 7) as i32;
                o.score += (o.x + o.y) as i64;
                cs = cs.wrapping_add((o.score ^ o.id) as u64);
                if o.hp <= 0 {
                    cs = cs.wrapping_add(o.score as u64);
                    let r2 = rng.next();
                    let fresh = place(&mut slabs, new_obj(next_id, i as u32, r2));
                    next_id += 1;
                    slabs[team].remove(h & !TEAM_BIT);
                    slots[i] = fresh;
                }
            }
        }
        _ => panic!("form: box | pbox | slab"),
    }
    cs = cs.wrapping_add(next_id as u64);
    harness::report(&format!("rust_churn_{}", form), t0, cs);
}
