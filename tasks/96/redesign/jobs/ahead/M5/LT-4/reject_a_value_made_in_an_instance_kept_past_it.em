# Clause LT-4.1: "A value made in a memory manager's instance (...), and every value that holds one,
# lives no longer than that instance, wherever it is kept: keeping or returning it where it would
# outlive the instance is `E3063` ("`boss` holds a mesh made in `current` (line 12), ...")."
#$ test: compile-fail
#$ rules: LT-4, ALC-1, BCK-4
# The first message is the spec's, with the file's line number; the second case (kept in the
# caller's list) has no message text in the spec, so it names the code only.
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

fn load_level() -> Enemy:
    current = LevelArena()
    boss = Enemy(hp=500, mesh=owns Mesh(points=[0.0, 1.0]) using current)
    return boss    #$ error[E3063]: `boss` holds a mesh made in `current` (line 34), and `current` ends when `load_level` returns, so `boss` cannot be returned; make it in an arena the caller keeps: `fn load_level(ref mut arena: LevelArena) -> Enemy:`, with `mesh=owns Mesh(points=[0.0, 1.0]) using arena`, and in the caller `current = LevelArena()` and `boss = load_level(ref mut current)`

fn stash(ref mut out: Array[Enemy]):
    current = LevelArena()
    out.push(owns Enemy(hp=1, mesh=owns Mesh(points=[0.0]) using current))    #$ error[E3063]

fn main():
    boss = load_level()
    kept: Array[Enemy] = []
    stash(ref mut kept)
    println(boss.hp, kept.len())
