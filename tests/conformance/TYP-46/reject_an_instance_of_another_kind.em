#$ test: compile-fail
#$ rules: TYP-46, ALC-1
## `[ALC-1]` — checked when compiling: the instance is of the kind the place names (`E2020`).

from std.mem import Allocator, Layout, Global, System

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

struct Enemy:
    mesh: owns Mesh using LevelArena

fn main():
    level = Arena.with_capacity(1024)
    boss = Enemy(mesh=owns Mesh(points=[0.0]) using level)    #$ error[E2020]: `mesh` takes its memory from a `LevelArena` (`mesh: owns Mesh using LevelArena`), and this `Mesh` comes from an `Arena`
    println(boss.mesh.points.len())
