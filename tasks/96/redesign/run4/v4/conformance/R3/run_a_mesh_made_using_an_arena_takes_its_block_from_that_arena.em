#$ test: run-pass
#$ rules: R3
#$ stdout: 1 2
#$ 1
## R3 and `V4/rules.md` section 2 ("the memory manager's own work"): the field names the kind
## (`owns Mesh using LevelArena`), the making line names the instance (`using current`), which is asked
## for the mesh's block once (`current.allocate(...)`); the mesh's own `verts` list comes from the
## built-in manager, as its field's type says; ending the level gives the block back to `current`
## (verdict 24; AN-40). `current` is read only after `lvl`'s end, so no view of it is in use then.
## Verified by its output and a clean AddressSanitizer run.

import std.mem

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

fn main():
    current = LevelArena()
    lvl = Level(mesh=owns Mesh(verts=[0, 1]) using current)
    n = lvl.mesh.verts.len()
    mem.drop(lvl)
    println(current.made, n)
    println(current.ended)
