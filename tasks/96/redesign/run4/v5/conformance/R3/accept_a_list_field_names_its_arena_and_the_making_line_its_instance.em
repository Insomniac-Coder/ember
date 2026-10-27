#$ test: run-pass
#$ rules: R3
#$ stdout: 0 1
## R3: the nearest program that is allowed: an `Array` has heap memory of its own, so its field may
## name the kind of arena, and the list is made in an instance (`[...] using current`).

from std.mem import Allocator, Layout, Global

struct LevelArena:
    made: int = 0
    ended: int = 0

# SAFETY: every block comes from `Global`, whose promises it keeps; this only counts the calls.
unsafe extend LevelArena implements Allocator:
    fn allocate(layout: Layout) -> *mut u8:
        this.made += 1
        g = Global()
        return g.allocate(layout)

    fn deallocate(p: *mut u8, layout: Layout):
        this.ended += 1
        g = Global()
        g.deallocate(p, layout)

    fn reallocate(p: *mut u8, old: Layout, new_size: usize) -> *mut u8:
        g = Global()
        return g.reallocate(p, old, new_size)

struct Names:
    count: int
    names: Array[String] using LevelArena

fn main():
    current = LevelArena()
    n = Names(count=0, names=[String("a")] using current)
    println(n.count, n.names.len())
