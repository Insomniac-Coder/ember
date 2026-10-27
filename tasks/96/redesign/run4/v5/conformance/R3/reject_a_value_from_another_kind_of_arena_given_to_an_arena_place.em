#$ test: compile-fail
#$ rules: R3
#$ help: make it in one: `lvl = Level(mesh=owns Mesh(verts=[0]) using current)`
## R3: the instance on the making line must be of the kind the place names (E2020; `V5/rules.md`
## section 3's second message, in this program's names).

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

struct FrameArena:
    made: int = 0

# SAFETY: every block comes from `Global`, whose promises it keeps; this only counts the calls.
unsafe extend FrameArena implements Allocator:
    fn allocate(layout: Layout) -> *mut u8:
        this.made += 1
        g = Global()
        return g.allocate(layout)

    fn deallocate(p: *mut u8, layout: Layout):
        g = Global()
        g.deallocate(p, layout)

    fn reallocate(p: *mut u8, old: Layout, new_size: usize) -> *mut u8:
        g = Global()
        return g.reallocate(p, old, new_size)

fn main():
    current = LevelArena()
    frame = FrameArena()
    lvl = Level(mesh=owns Mesh(verts=[0]) using frame)     #$ error[E2020]: `frame` is a `FrameArena`, and `mesh` takes its memory from a `LevelArena`
    println(lvl.mesh.verts.len())
