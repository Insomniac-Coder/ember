#$ test: compile-fail
#$ rules: R3
#$ help: say which arena: `lvl = Level(mesh=owns Mesh(verts=[0]) using current)`
## R3: a value given to a `using A` place must have been made in an instance of `A` (E2020); the
## message says which arena to name (verdict 24; AN-40's message). `owns` is written on the making line
## (verdict 74), so the arena is the line's one fault.

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
    lvl = Level(mesh=owns Mesh(verts=[0]))     #$ error[E2020]: `mesh` takes its memory from a `LevelArena` (`mesh: owns Mesh using LevelArena`), and this `Mesh` comes from the built-in manager
    println(lvl.mesh.verts.len())
