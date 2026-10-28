#$ test: run-pass
#$ rules: R3, R7
#$ stdout: 1
## R3, R7: the nearest program that is allowed: the caller makes the arena and keeps it longer than
## the level (AN-40's help).

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

fn load_level(ref mut arena: LevelArena) -> Level:
    lvl = Level(mesh=owns Mesh(verts=[0]) using arena)
    return lvl

fn main():
    current = LevelArena()
    lvl = load_level(ref mut current)
    println(lvl.mesh.verts.len())
