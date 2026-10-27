//! `[BUD-2]`'s B9, ODR-281 — what the compiler holds, stage by stage. With the `count-allocations`
//! feature (`cargo build --profile ci -p ember_driver --features count-allocations`) every
//! allocation goes through a counter, and `--timings` gives each stage the bytes live at its end
//! and the most live during it; without it nothing is counted and nothing costs.

/// The bytes live now and the most live since the last call, in megabytes; `None` unless the
/// compiler was built to count.
pub(crate) fn take() -> Option<(f64, f64)> {
    counting::take()
}

#[cfg(feature = "count-allocations")]
#[allow(unsafe_code)]
mod counting {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

    static LIVE: AtomicUsize = AtomicUsize::new(0);
    static PEAK: AtomicUsize = AtomicUsize::new(0);

    struct Counting;

    #[global_allocator]
    static COUNTING: Counting = Counting;

    fn grew(bytes: usize) {
        let now = LIVE.fetch_add(bytes, Relaxed) + bytes;
        PEAK.fetch_max(now, Relaxed);
    }

    // SAFETY(ffi): each call is the system allocator's own, with the layout it was given; the
    // counters only add and subtract the sizes it was asked for.
    unsafe impl GlobalAlloc for Counting {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            // SAFETY(ffi): forwarded as given.
            let pointer = unsafe { System.alloc(layout) };
            if !pointer.is_null() {
                grew(layout.size());
            }
            pointer
        }
        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            // SAFETY(ffi): forwarded as given.
            let pointer = unsafe { System.alloc_zeroed(layout) };
            if !pointer.is_null() {
                grew(layout.size());
            }
            pointer
        }
        unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
            // SAFETY(ffi): forwarded as given.
            unsafe { System.dealloc(pointer, layout) };
            LIVE.fetch_sub(layout.size(), Relaxed);
        }
        unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
            // SAFETY(ffi): forwarded as given.
            let moved = unsafe { System.realloc(pointer, layout, size) };
            if !moved.is_null() {
                grew(size);
                LIVE.fetch_sub(layout.size(), Relaxed);
            }
            moved
        }
    }

    pub(super) fn take() -> Option<(f64, f64)> {
        let live = LIVE.load(Relaxed);
        let peak = PEAK.swap(live, Relaxed).max(live);
        let mb = |bytes: usize| bytes as f64 / (1024.0 * 1024.0);
        Some((mb(live), mb(peak)))
    }
}

#[cfg(not(feature = "count-allocations"))]
mod counting {
    pub(super) fn take() -> Option<(f64, f64)> {
        None
    }
}
