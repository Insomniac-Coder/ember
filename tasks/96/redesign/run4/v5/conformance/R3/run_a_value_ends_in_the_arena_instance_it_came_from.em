#$ test: run-pass
#$ rules: R3
#$ stdout: 1
#$ 1 0
## R3 and `V5/rules.md` section 2 ("one word in the value: its instance's address"): a `LevelArena`
## may have several instances, so each value made in one keeps that instance's address, and its end
## goes back to the instance it came from, not to another of the same kind (verdict 47). The counts
## are read after `b`'s last use.

import std.mem

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

struct Mesh:
    verts: Array[int]

struct Level:
    mesh: owns Mesh using LevelArena

fn main():
    current = LevelArena()
    next = LevelArena()
    a = Level(mesh=owns Mesh(verts=[0]) using current)
    b = Level(mesh=owns Mesh(verts=[1]) using next)
    mem.drop(a)
    println(b.mesh.verts.len())
    println(current.ended, next.ended)
