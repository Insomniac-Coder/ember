#$ test: compile-fail
#$ rules: MOD-3
#$ help: write `names: Array[String] using LevelArena`
## design rules: R3
## R3 and the removed words of `V5/rules.md` section 3: `Array[T, A]` is written `Array[T] using A` (E1010; the
## `using` row of `V5/vocabulary.md`; completeness gap G8). The nearest program that is allowed is
## `accept_a_list_field_names_its_arena_and_the_making_line_its_instance`.

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

struct Names:
    names: Array[String, LevelArena]    #$ error[E1010]: an `Array`'s memory manager is not a type parameter

fn main():
    println(0)
