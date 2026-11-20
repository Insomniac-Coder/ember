# Clause LT-4.4: "A manager with one instance (`Global`, `System`) costs no word."
#$ test: run-pass
#$ rules: LT-4, ALC-1
#$ stdout: true true true
# A value made in `Global` or `System` is a pointer and nothing more; one made in a `LevelArena`
# (several instances, a `deallocate` that gives blocks back, not `FoundByAddress`) keeps one word
# more, its instance's address.
from std.mem import Allocator, Layout, Global, System

struct LevelArena:
    made: int = 0

# SAFETY: every block comes from `Global`, which keeps the promises; this only counts the blocks.
unsafe extend LevelArena implements Allocator:
    fn allocate(layout: Layout) -> *mut u8:
        this.made += 1
        return Global().allocate(layout)

    @safety("`p` came from this arena's `allocate` with `layout`, and nothing uses it after this call")
    unsafe fn deallocate(p: *mut u8, layout: Layout):
        Global().deallocate(p, layout)

    @safety("`p` came from this arena's `allocate` with `old`, and nothing uses it after this call")
    unsafe fn reallocate(p: *mut u8, old: Layout, new_size: usize) -> *mut u8:
        return Global().reallocate(p, old, new_size)

struct Mesh:
    points: Array[float]

struct Enemy:
    hp: int
    mesh: owns Mesh using LevelArena

struct InDefault:
    mesh: owns Mesh

struct InGlobal:
    mesh: owns Mesh using Global

struct InSystem:
    mesh: owns Mesh using System

struct InLevel:
    mesh: owns Mesh using LevelArena

fn main():
    word = size_of[usize]()
    println(size_of[InDefault]() == word, size_of[InGlobal]() == word and size_of[InSystem]() == word, size_of[InLevel]() == 2 * word)
