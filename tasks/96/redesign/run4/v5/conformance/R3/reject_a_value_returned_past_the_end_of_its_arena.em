#$ test: compile-fail
#$ rules: R3, R7
#$ help: make it in an arena the caller keeps: `fn load_level(ref mut arena: LevelArena) -> Level:`, with `lvl = Level(mesh=owns Mesh(verts=[0]) using arena)`, and in the caller `current = LevelArena()` and `lvl = load_level(ref mut current)`
## R3, R7: a value made in a memory manager's instance lives no longer than that instance (E3063);
## the message moves the arena to the caller (AN-40's refused `load_level` and message).

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

fn load_level() -> Level:
    current = LevelArena()
    lvl = Level(mesh=owns Mesh(verts=[0]) using current)
    return lvl                 #$ error[E3063]: `lvl` holds a mesh made in `current` (line 37), and `current` ends when `load_level` returns, so `lvl` cannot be returned (line 38)

fn main():
    lvl = load_level()
    println(lvl.mesh.verts.len())
