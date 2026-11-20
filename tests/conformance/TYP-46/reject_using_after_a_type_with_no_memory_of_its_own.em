#$ test: compile-fail
#$ rules: TYP-46, ALC-1
## `[TYP-46]` — `using` after a type with no heap memory of its own (an `int`, a struct held inline,
## a `FixedArray`) is `E2114`: "an `int` has no memory of its own to take from `LevelArena`; remove
## `using`".

from std.mem import Allocator, Layout, Global

struct LevelArena:
    made: int = 0

# SAFETY: every block comes from `Global`, which keeps the promises.
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

struct Point:
    x: int
    y: int

struct Level:
    count: int using LevelArena    #$ error[E2114]: an `int` has no memory of its own to take from `LevelArena`; remove `using`
    at: Point using LevelArena    #$ error[E2114]: a `Point` has no memory of its own to take from `LevelArena`; remove `using`
    cells: [int; 4] using LevelArena    #$ error[E2114]: has no memory of its own to take from `LevelArena`; remove `using`

fn main():
    pass
