# Clause LT-4.1 (its way out, `[DIA-25]`: the line a message gives compiles): "make it in an arena
# the caller keeps: `fn load_level(ref mut arena: LevelArena) -> Enemy:`, with `mesh=owns
# Mesh(points=[0.0, 1.0]) using arena`, and in the caller `current = LevelArena()` and `boss =
# load_level(ref mut current)`"
#$ test: run-pass
#$ rules: LT-4, ALC-1, DIA-25
#$ stdout: 500 2
from std.mem import Allocator, Layout, Global

struct LevelArena:
    made: int = 0

# SAFETY: every block comes from `Global`, which keeps the promises; this only counts the blocks.
unsafe extend LevelArena implements Allocator:
    fn allocate(layout: Layout) -> *mut u8:
        this.made += 1
        return Global().allocate(layout)

    unsafe fn deallocate(p: *mut u8, layout: Layout):
        Global().deallocate(p, layout)

    unsafe fn reallocate(p: *mut u8, old: Layout, new_size: usize) -> *mut u8:
        return Global().reallocate(p, old, new_size)

struct Mesh:
    points: Array[float]

struct Enemy:
    hp: int
    mesh: owns Mesh using LevelArena

fn load_level(ref mut arena: LevelArena) -> Enemy:
    boss = Enemy(hp=500, mesh=owns Mesh(points=[0.0, 1.0]) using arena)
    return boss

fn main():
    current = LevelArena()
    boss = load_level(ref mut current)
    println(boss.hp, boss.mesh.points.len())
