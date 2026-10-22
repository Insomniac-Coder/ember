#$ test: run-pass
#$ rules: ALC-2, TYP-43, UNS-1
#$ profiles: debug, development, release
#$ stdout: true true true 3
## `[ALC-2]` (ODR-175) — `std.mem.Allocator` is an `unsafe interface`: `allocate(self, layout:
## Layout) -> *mut u8` (null when there is no memory), `deallocate(self, p, layout)` and
## `reallocate(self, p, old, new_size) -> *mut u8`. Implementing it is `unsafe`
## (`unsafe extend T implements Allocator:`, `[TYP-43]` as amended), its implementer promising
## valid, aligned, unaliased memory. `Global` is the runtime's, `System` the platform's (ODR-239).
## A generic function bounded by `Allocator` takes any of them.

from std.mem import Allocator, Layout, Global, System
from std.sync import Atomic

struct Counting:
    calls: Atomic[int]

# SAFETY: every block comes from `Global`, whose promises it keeps; this only counts the calls.
unsafe extend Counting implements Allocator:
    fn allocate(self, layout: Layout) -> *mut u8:
        self.calls.fetch_add(1)
        return Global().allocate(layout)

    fn deallocate(self, p: *mut u8, layout: Layout):
        self.calls.fetch_add(1)
        Global().deallocate(p, layout)

    fn reallocate(self, p: *mut u8, old: Layout, new_size: usize) -> *mut u8:
        self.calls.fetch_add(1)
        return Global().reallocate(p, old, new_size)

fn round_trip[A: Allocator](a: A) -> bool:
    p = a.allocate(Layout(16, 8))
    if p == null[*mut u8]():
        return false
    q = a.reallocate(p, Layout(16, 8), 128)
    a.deallocate(q, Layout(128, 8))
    return q != null[*mut u8]()

fn main():
    counting = Counting(Atomic(0))
    println(round_trip(Global()), round_trip(System()), round_trip(counting), counting.calls.load())
