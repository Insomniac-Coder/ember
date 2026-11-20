#$ test: run-pass
#$ rules: TYP-46, ALC-1
#$ stdout: 2 2
## `[ALC-1]` — `using x` names the instance through a name that may change it (its owner, a `ref mut`
## name) where its `allocate` changes the instance, and through any name where its `allocate` only
## reads it: a `static` instance, whose count is an `Atomic`.

from std.mem import Allocator, Layout, Global
from std.sync import Atomic

struct LevelArena:
    made: int = 0

# SAFETY: every block comes from `Global`, which keeps the promises; this only counts the blocks.
unsafe extend LevelArena implements Allocator:
    fn allocate(layout: Layout) -> *mut u8:
        this.made += 1
        return Global().allocate(layout)

    @safety("`p` came from this arena's `allocate` with `layout`")
    unsafe fn deallocate(p: *mut u8, layout: Layout):
        Global().deallocate(p, layout)

    @safety("`p` came from this arena's `allocate` with `old`")
    unsafe fn reallocate(p: *mut u8, old: Layout, new_size: usize) -> *mut u8:
        return Global().reallocate(p, old, new_size)

struct Shared:
    made: Atomic[int]

# SAFETY: every block comes from `Global`; the count is atomic, so `allocate` only reads its object.
unsafe extend Shared implements Allocator:
    fn allocate(layout: Layout) -> *mut u8:
        this.made.fetch_add(1)
        return Global().allocate(layout)

    @safety("`p` came from this manager's `allocate` with `layout`")
    unsafe fn deallocate(p: *mut u8, layout: Layout):
        Global().deallocate(p, layout)

    @safety("`p` came from this manager's `allocate` with `old`")
    unsafe fn reallocate(p: *mut u8, old: Layout, new_size: usize) -> *mut u8:
        return Global().reallocate(p, old, new_size)

static HEAP: Shared = Shared(Atomic(0))

struct Cmd:
    id: int

fn make(ref mut current: LevelArena, id: int) -> owns Cmd using LevelArena:
    return owns Cmd(id=id) using current

fn make_shared(id: int) -> owns Cmd using Shared:
    return owns Cmd(id=id) using HEAP

fn main():
    current = LevelArena()
    a = make(ref mut current, 1)
    b = make(ref mut current, 2)
    c = make_shared(3)
    d = make_shared(4)
    println(current.made, HEAP.made.load())
    mem.drop(a)
    mem.drop(b)
    mem.drop(c)
    mem.drop(d)
