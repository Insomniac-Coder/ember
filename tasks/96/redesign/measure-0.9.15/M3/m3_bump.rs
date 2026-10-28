// m3_bump.rs: the hand-written arena M3's Rust programs share (no crates: plain rustc).
//
// `Bump` is the runtime arena of m3_rt.h written the way Rust's arena crates write it (bumpalo):
// chunks of 1 MB, kept by `reset`, which takes `&mut self` so the borrow checker proves nothing
// made in the arena is still used. `unsafe` is used inside it only, as in those crates.
//
// Two owners of arena memory, as stable Rust can spell them (`Box<T, &A>` and `Vec<T, &A>` need
// the unstable allocator_api):
// - `ABox<'a, T>`: a block and the arena's address, 16 bytes: exactly `Box<T, &'a Bump>`'s layout
//   and behaviour (its drop runs T's drop, then gives the block back through the kept `&Bump`);
// - `AVec<'a, T>`: pointer, length, capacity and the arena's address, 32 bytes: `Vec<T, &'a Bump>`
//   and `bumpalo::collections::Vec<'a, T>` keep the same four words.
// bumpalo's own `Box<'a, T>` is a `&'a mut T` with no arena word (it never gives a block back):
// the `&'a T` links of rust_trees.rs's `bref` form are that idiom.
#![allow(dead_code)]
use std::alloc::{alloc, dealloc, Layout};
use std::cell::{Cell, RefCell};
use std::ops::{Deref, DerefMut};
use std::ptr::{self, NonNull};

pub const CHUNK: usize = 1 << 20;
const CHUNK_ALIGN: usize = 16;

pub struct Bump {
    chunks: RefCell<Vec<(*mut u8, usize)>>, // every chunk in order, kept by reset
    idx: Cell<usize>,                        // the current chunk
    cur: Cell<usize>,
    end: Cell<usize>,
}

impl Bump {
    pub fn new() -> Bump {
        let b = Bump { chunks: RefCell::new(Vec::new()), idx: Cell::new(0), cur: Cell::new(0), end: Cell::new(0) };
        b.add_chunk(CHUNK);
        b
    }

    fn add_chunk(&self, size: usize) {
        let layout = Layout::from_size_align(size, CHUNK_ALIGN).expect("chunk layout");
        let base = unsafe { alloc(layout) };
        if base.is_null() {
            std::alloc::handle_alloc_error(layout);
        }
        let mut chunks = self.chunks.borrow_mut();
        chunks.push((base, size));
        self.idx.set(chunks.len() - 1);
        self.cur.set(base as usize);
        self.end.set(base as usize + size);
    }

    /// The bump, inline: the current chunk has room, else the slow path.
    #[inline(always)]
    pub fn alloc_raw(&self, size: usize, align: usize) -> *mut u8 {
        let p = (self.cur.get() + align - 1) & !(align - 1);
        if p <= self.end.get() && size <= self.end.get() - p {
            self.cur.set(p + size);
            return p as *mut u8;
        }
        self.slow(size, align)
    }

    /// The next kept chunk with room (the runtime's walk), else a new one.
    #[inline(never)]
    #[cold]
    fn slow(&self, size: usize, align: usize) -> *mut u8 {
        let n = self.chunks.borrow().len();
        let mut i = self.idx.get() + 1;
        while i < n {
            let (base, sz) = self.chunks.borrow()[i];
            let start = base as usize;
            let p = (start + align - 1) & !(align - 1);
            if p + size <= start + sz {
                self.idx.set(i);
                self.cur.set(p + size);
                self.end.set(start + sz);
                return p as *mut u8;
            }
            i += 1;
        }
        let need = size.checked_add(align).expect("arena allocation overflow");
        self.add_chunk(if need > CHUNK { need } else { CHUNK });
        let p = (self.cur.get() + align - 1) & !(align - 1);
        self.cur.set(p + size);
        p as *mut u8
    }

    /// bumpalo's `alloc`: the value moved into the arena, a view of it for the arena's borrow.
    #[inline(always)]
    pub fn alloc<T>(&self, v: T) -> &mut T {
        let p = self.alloc_raw(std::mem::size_of::<T>(), std::mem::align_of::<T>()) as *mut T;
        unsafe {
            p.write(v);
            &mut *p
        }
    }

    /// `Allocator::grow`: in place when `p` is the last block of the current chunk.
    #[inline(always)]
    pub fn grow(&self, p: *mut u8, old: usize, new: usize, align: usize) -> *mut u8 {
        if !p.is_null() && p as usize + old == self.cur.get() && new - old <= self.end.get() - self.cur.get() {
            self.cur.set(self.cur.get() + (new - old));
            return p;
        }
        let q = self.alloc_raw(new, align);
        if old != 0 {
            unsafe { ptr::copy_nonoverlapping(p, q, old) };
        }
        q
    }

    /// `Allocator::deallocate`: an arena frees nothing one by one.
    #[inline(always)]
    pub fn dealloc(&self, _p: *mut u8, _size: usize) {}

    pub fn reset(&mut self) {
        let chunks = self.chunks.borrow();
        let (base, sz) = chunks[0];
        self.idx.set(0);
        self.cur.set(base as usize);
        self.end.set(base as usize + sz);
    }
}

impl Drop for Bump {
    fn drop(&mut self) {
        for &(base, sz) in self.chunks.borrow().iter() {
            unsafe { dealloc(base, Layout::from_size_align_unchecked(sz, CHUNK_ALIGN)) };
        }
    }
}

/// `Box<T, &'a Bump>`: the block and the arena's address.
pub struct ABox<'a, T> {
    p: NonNull<T>,
    a: &'a Bump,
}

impl<'a, T> ABox<'a, T> {
    #[inline(always)]
    pub fn new_in(v: T, a: &'a Bump) -> ABox<'a, T> {
        let raw = a.alloc_raw(std::mem::size_of::<T>(), std::mem::align_of::<T>()) as *mut T;
        let p = match NonNull::new(raw) {
            Some(p) => p,
            None => std::alloc::handle_alloc_error(Layout::new::<T>()),
        };
        unsafe { p.as_ptr().write(v) };
        ABox { p, a }
    }
}

impl<'a, T> Deref for ABox<'a, T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        unsafe { self.p.as_ref() }
    }
}

impl<'a, T> Drop for ABox<'a, T> {
    #[inline]
    fn drop(&mut self) {
        unsafe { ptr::drop_in_place(self.p.as_ptr()) };
        self.a.dealloc(self.p.as_ptr() as *mut u8, std::mem::size_of::<T>());
    }
}

/// `Vec<T, &'a Bump>`: growth and the end go back to the kept arena.
pub struct AVec<'a, T> {
    ptr: *mut T,
    len: usize,
    cap: usize,
    a: &'a Bump,
}

impl<'a, T> AVec<'a, T> {
    #[inline(always)]
    pub fn new_in(a: &'a Bump) -> AVec<'a, T> {
        AVec { ptr: NonNull::dangling().as_ptr(), len: 0, cap: 0, a }
    }

    pub fn with_capacity_in(n: usize, a: &'a Bump) -> AVec<'a, T> {
        let mut v = AVec::new_in(a);
        v.grow_to(n);
        v
    }

    #[inline(never)]
    fn grow_to(&mut self, new_cap: usize) {
        let sz = std::mem::size_of::<T>();
        let new_bytes = new_cap.checked_mul(sz).filter(|&b| b <= isize::MAX as usize).expect("capacity overflow");
        let old_ptr = if self.cap == 0 { ptr::null_mut() } else { self.ptr as *mut u8 };
        let q = self.a.grow(old_ptr, self.cap * sz, new_bytes, std::mem::align_of::<T>());
        if q.is_null() {
            std::alloc::handle_alloc_error(Layout::new::<T>());
        }
        self.ptr = q as *mut T;
        self.cap = new_cap;
    }

    #[inline(always)]
    pub fn push(&mut self, v: T) {
        if self.len == self.cap {
            let nc = if self.cap == 0 { 4 } else { self.cap * 2 };
            self.grow_to(nc);
        }
        unsafe { self.ptr.add(self.len).write(v) };
        self.len += 1;
    }
}

impl<'a, T> Deref for AVec<'a, T> {
    type Target = [T];
    #[inline(always)]
    fn deref(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
}

impl<'a, T> DerefMut for AVec<'a, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}

impl<'a, T> Drop for AVec<'a, T> {
    #[inline]
    fn drop(&mut self) {
        unsafe { ptr::drop_in_place(std::slice::from_raw_parts_mut(self.ptr, self.len)) };
        if self.cap != 0 {
            self.a.dealloc(self.ptr as *mut u8, self.cap * std::mem::size_of::<T>());
        }
    }
}
