#$ test: run-pass
#$ rules: TYP-46, ALC-1, ALC-2
#$ stdout: true true true true
#$ stdout: true true
## `[ALC-1]` (verdict 107) — a value whose manager may have several instances keeps its instance's
## address, one word; one made in `Global` or `System` (a manager with one instance) keeps none, nor
## one made in a manager that frees nothing one by one (an `Arena`, whose `deallocate` does nothing),
## nor one in a manager that implements `FoundByAddress` (`[ALC-2]`). `owns T using Global` is `owns T`.

from std.mem import Allocator, FoundByAddress, Layout, Global, System

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

struct Pieces:
    first: *mut u8

# SAFETY: it hands out no block at all.
unsafe extend Pieces implements Allocator:
    fn allocate(layout: Layout) -> *mut u8:
        return null[*mut u8]()

    @safety("`p` came from this manager's `allocate`")
    unsafe fn deallocate(p: *mut u8, layout: Layout):
        Global().deallocate(p, layout)

    @safety("`p` came from this manager's `allocate`")
    unsafe fn reallocate(p: *mut u8, old: Layout, new_size: usize) -> *mut u8:
        return null[*mut u8]()

# SAFETY: it hands out no block, so each lies in a piece.
unsafe extend Pieces implements FoundByAddress:
    pass

struct Mesh:
    points: Array[float]

struct InDefault:
    mesh: owns Mesh

struct InGlobal:
    mesh: owns Mesh using Global

struct InSystem:
    mesh: owns Mesh using System

struct InLevel:
    mesh: owns Mesh using LevelArena

struct InArena:
    mesh: owns Mesh using Arena

struct InPieces:
    mesh: owns Mesh using Pieces

fn main():
    word = size_of[usize]()
    println(size_of[InDefault]() == word, size_of[InGlobal]() == word, size_of[InSystem]() == word, size_of[InLevel]() == 2 * word)
    println(size_of[InArena]() == word, size_of[InPieces]() == word)
    a = InDefault(mesh=owns Mesh(points=[1.0]))
    b = InGlobal(mesh=owns a.mesh)
    mem.drop(b)
