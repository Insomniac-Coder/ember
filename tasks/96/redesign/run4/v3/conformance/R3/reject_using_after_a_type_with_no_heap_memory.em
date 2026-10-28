#$ test: compile-fail
#$ rules: R3
#$ help: remove `using`
## R3: `using A` after a type whose value has no heap memory of its own is refused (E2114;
## `V3/rules.md` section 3's message).

from std.mem import Allocator, Layout, Global

struct LevelArena:
    made: int = 0
    ended: int = 0

# SAFETY: every block comes from `Global`, whose promises it keeps; this only counts the calls.
unsafe extend LevelArena implements Allocator:
    fn allocate(layout: Layout) changes this -> *mut u8:
        this.made += 1
        g = Global()
        return g.allocate(layout)

    fn deallocate(p: *mut u8, layout: Layout) changes this:
        this.ended += 1
        g = Global()
        g.deallocate(p, layout)

    fn reallocate(p: *mut u8, old: Layout, new_size: usize) changes this -> *mut u8:
        g = Global()
        return g.reallocate(p, old, new_size)

struct Mesh:
    verts: Array[int]

struct Level:
    mesh: owns Mesh using LevelArena

struct Stats:
    count: int using LevelArena    #$ error[E2114]: an `int` has no memory of its own to take from `LevelArena`

fn main():
    s = Stats(count=0)
    println(s.count)
