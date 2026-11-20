#$ test: compile-fail
#$ rules: TYP-46, ALC-1
## `[TYP-46]`, `[ALC-1]` — a value given to a `using A` place that was not made in an instance of `A`
## is `E2020`: a place with no `using` is a `using Global` place, so a place's type always names its
## values' manager.

from std.mem import Allocator, Layout, Global

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
    hp: int
    mesh: owns Mesh using LevelArena

struct Plain:
    mesh: owns Mesh

fn main():
    current = LevelArena()
    boss = Enemy(hp=500, mesh=owns Mesh(points=[0.0, 1.0]))    #$ error[E2020]: `mesh` takes its memory from a `LevelArena` (`mesh: owns Mesh using LevelArena`), and this `Mesh` comes from the built-in manager; say which arena: `mesh=owns Mesh(points=[0.0, 1.0]) using current`
    made = owns Mesh(points=[1.0]) using current
    plain = Plain(mesh=made)    #$ error[E2020]: `mesh` takes its memory from the built-in manager (`mesh: owns Mesh`), and this `Mesh` comes from a `LevelArena`
    println(boss.hp, plain.mesh.points.len())
