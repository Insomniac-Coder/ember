#$ test: run-pass
#$ rules: TYP-46, ALC-1
#$ stdout: 1 2 2
## `[TYP-46]`, `[ALC-1]` — a manager with one instance (`Global`, `System`, and any manager type with
## no fields a program declares) needs none on the making line: `owns Mesh(points=[1.0])` for an
## `owns Mesh using System` place takes its block from `System`. It may still be named.

from std.mem import Allocator, Layout, System
from std.sync import Atomic

static MADE: Atomic[int] = Atomic(0)

struct Counted:
    pass

# SAFETY: every block comes from `System`; this only counts them.
unsafe extend Counted implements Allocator:
    fn allocate(layout: Layout) -> *mut u8:
        MADE.fetch_add(1)
        return System().allocate(layout)

    @safety("`p` came from this manager's `allocate` with `layout`")
    unsafe fn deallocate(p: *mut u8, layout: Layout):
        System().deallocate(p, layout)

    @safety("`p` came from this manager's `allocate` with `old`")
    unsafe fn reallocate(p: *mut u8, old: Layout, new_size: usize) -> *mut u8:
        return System().reallocate(p, old, new_size)

struct Mesh:
    points: Array[float]

fn main():
    a: owns Mesh using System = owns Mesh(points=[1.0])
    b: owns Mesh using Counted = owns Mesh(points=[1.0, 2.0])
    counted = Counted()
    c = owns Mesh(points=[1.0, 2.0]) using counted
    println(a.points.len(), b.points.len(), MADE.load())
    mem.drop(c)
