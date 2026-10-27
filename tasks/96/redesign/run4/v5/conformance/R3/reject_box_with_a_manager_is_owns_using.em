#$ test: compile-fail
#$ rules: MOD-3
#$ help: write `mesh: owns Mesh using LevelArena`, and on the line that makes the value name the arena: `lvl = Level(mesh=owns Mesh(verts=[0]) using current)`
## design rules: R3
## R3 and the removed words of `V5/rules.md` section 3: `Box[T, A]` is written `owns T using A` (E1010; the `Box`
## row of `V5/vocabulary.md`; completeness gap G8). The nearest program that is allowed is
## `accept_a_level_made_in_an_arena_the_caller_keeps`.

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
    mesh: Box[Mesh, LevelArena]    #$ error[E1010]: `Box` is not part of Ember

fn main():
    println(0)
