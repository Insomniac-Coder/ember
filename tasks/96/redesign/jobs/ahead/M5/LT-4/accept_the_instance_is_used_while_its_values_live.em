# Clause LT-4.3: "A value whose manager may have several instances keeps its instance's address, one
# word, so that its growth and its end go back to the instance it came from; that address is a kept
# `sharedref mut` link to the instance (`[LT-46]`) for the exclusivity rules, and between threads
# what `[TYP-46a]` says (...): the program may use the instance while values made in it live, and
# each value's growth and end change the instance through it."
#$ test: run-pass
#$ rules: LT-4, ALC-1, ALC-5, LT-46
#$ stdout: true 2 1
# `names` grows (its growth goes back to `current`), another value is made in `current`, and
# `current` is read, all while `names` lives.
from std.mem import Allocator, Layout, Global

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

fn main():
    current = LevelArena()
    names: Array[String] using LevelArena = [String("a")] using current
    names.push(owns String("b"))
    other: Array[float] using LevelArena = [1.0] using current
    println(current.made > 0, names.len(), other.len())
