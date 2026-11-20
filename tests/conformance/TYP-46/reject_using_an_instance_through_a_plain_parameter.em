#$ test: compile-fail
#$ rules: ALC-1
## `[ALC-1]` — `using x` through a plain parameter of a manager whose `allocate` changes it is
## `E3023`, with the line to write.

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

struct Mesh:
    points: Array[float]

fn fill(current: LevelArena) -> owns Mesh using LevelArena:
    return owns Mesh(points=[0.0]) using current    #$ error[E3023]: `allocate` changes the arena (`LevelArena.allocate`, its line 13), and `current` only reads it; write `fn fill(ref mut current: LevelArena) -> owns Mesh using LevelArena`

fn main():
    current = LevelArena()
    m = fill(current)
    println(m.points.len())
