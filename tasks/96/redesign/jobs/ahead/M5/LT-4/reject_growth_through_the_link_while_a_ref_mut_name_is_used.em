# Clause LT-4.3: "that address is a kept `sharedref mut` link to the instance (`[LT-46]`) for the
# exclusivity rules"
#$ test: compile-fail
#$ rules: LT-4, LT-46, BRW-1
# `r` is the only name that may change `current` while it is used (`[BRW-1]`: "a link of the same
# type included"); `names.push` changes `current` through the value's link (its growth goes back to
# the instance). The spec gives no message text for this case: the code only.
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
    r = ref mut current
    names.push(owns String("b"))    #$ error[E3022]
    r.made += 1
    println(names.len())
