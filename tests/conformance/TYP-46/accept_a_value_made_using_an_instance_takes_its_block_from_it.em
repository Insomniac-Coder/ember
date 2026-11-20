#$ test: run-pass
#$ rules: TYP-46, ALC-1, ALC-2, OWN-10, HEAP-1
#$ profiles: debug, development, release
#$ stdout: 2 1 0
#$ stdout: 1 1
## `[TYP-46]`, `[ALC-1]` — `using A` after an `owns` type names the kind of manager its block comes
## from; the line that makes the value names the instance after it, `owns Mesh(points=[0.0, 1.0])
## using current`, and takes the block by calling `current.allocate(...)`; the value's end gives it
## back through `current.deallocate(...)`. These calls are the compiler's own (`[ALC-2]`): the
## program writes no `unsafe:` for them. `using` names where the mesh's own block comes from; its
## `points` list comes from the built-in manager, as its type says.

from std.mem import Allocator, Layout, Global

struct LevelArena:
    made: int = 0
    given_back: int = 0

# SAFETY: every block comes from `Global`, which keeps the promises; this only counts the blocks.
unsafe extend LevelArena implements Allocator:
    fn allocate(layout: Layout) -> *mut u8:
        this.made += 1
        return Global().allocate(layout)

    @safety("`p` came from this arena's `allocate` with `layout`, and nothing uses it after this call")
    unsafe fn deallocate(p: *mut u8, layout: Layout):
        this.given_back += 1
        Global().deallocate(p, layout)

    @safety("`p` came from this arena's `allocate` with `old`, and nothing uses it after this call")
    unsafe fn reallocate(p: *mut u8, old: Layout, new_size: usize) -> *mut u8:
        return Global().reallocate(p, old, new_size)

struct Mesh:
    points: Array[float]

struct Enemy:
    hp: int
    mesh: owns Mesh using LevelArena

fn main():
    current = LevelArena()
    boss = Enemy(hp=500, mesh=owns Mesh(points=[0.0, 1.0]) using current)
    println(boss.mesh.points.len(), current.made, current.given_back)
    mem.drop(boss)
    println(current.made, current.given_back)
