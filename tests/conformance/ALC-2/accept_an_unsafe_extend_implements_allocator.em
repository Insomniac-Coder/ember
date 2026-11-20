#$ test: run-pass
#$ rules: ALC-2, TYP-43, UNS-1
#$ profiles: debug, development, release
#$ stdout: true true true 3
## `[ALC-2]` — `std.mem.Allocator` is an `unsafe interface`: `fn allocate(layout: Layout) -> *mut u8`
## (null when there is no memory), `unsafe fn deallocate(p, layout)` and `unsafe fn reallocate(p,
## old, new_size) -> *mut u8`. Implementing it is `unsafe` (`unsafe extend T implements Allocator:`,
## `[TYP-43]`), its implementer promising valid, aligned, unaliased memory. `Global` is the
## package's configured allocator and `System` the platform's, which a counting allocator wraps.
## A generic function bounded by `Allocator` takes any of them; giving memory back, or moving it,
## is its own call, made inside `unsafe:`.

from std.mem import Allocator, Layout, Global, System
from std.sync import Atomic

struct Counting:
    calls: Atomic[int]

# SAFETY: every block comes from `System`, whose promises it keeps; this only counts the calls.
unsafe extend Counting implements Allocator:
    fn allocate(layout: Layout) -> *mut u8:
        this.calls.fetch_add(1)
        return System().allocate(layout)

    @safety("`p` came from this manager's `allocate` or `reallocate` with `layout`, and nothing uses it after")
    unsafe fn deallocate(p: *mut u8, layout: Layout):
        this.calls.fetch_add(1)
        System().deallocate(p, layout)

    @safety("`p` came from this manager's `allocate` or `reallocate` with `old`, and nothing uses it after; only the result is used")
    unsafe fn reallocate(p: *mut u8, old: Layout, new_size: usize) -> *mut u8:
        this.calls.fetch_add(1)
        return System().reallocate(p, old, new_size)

[A: needs Allocator]
fn round_trip(a: A) -> bool:
    p = a.allocate(Layout(16, 8))
    if p == null[*mut u8]():
        return false
    q = null[*mut u8]()
    # SAFETY: `p` came from `a` with this layout, and nothing else uses it.
    unsafe:
        q = a.reallocate(p, Layout(16, 8), 128)
        a.deallocate(q, Layout(128, 8))
    return q != null[*mut u8]()

fn main():
    counting = Counting(Atomic(0))
    println(round_trip(Global()), round_trip(System()), round_trip(counting), counting.calls.load())
