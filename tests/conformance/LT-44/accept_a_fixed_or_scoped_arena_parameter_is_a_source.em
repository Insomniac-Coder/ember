#$ test: run-pass
#$ rules: LT-44, LT-1, ARN-1
#$ stdout: 1
#$ 2
#$ 3
# `[LT-44]` — a `ref mut` `FixedArena` or `ScopedArena` parameter is a source parameter, as an
# `Arena` is: a function whose only source parameter is one arena returns views that point into it,
# with no word, and `from arena` says the same. `alloc` goes through a name that may change the
# arena (`[ARN-1]`).

fn from_fixed(ref mut arena: FixedArena) -> ref mut i32:
    return arena.alloc(1)

fn from_named(ref mut arena: FixedArena) -> ref mut i32 from arena:
    return arena.alloc(2)

fn from_scoped(ref mut arena: ScopedArena) -> ref mut i32:
    return arena.alloc(3)

fn main():
    bytes: [u8; 64] = [0; 64]
    fixed: FixedArena = Arena.fixed(bytes)
    a = from_fixed(ref mut fixed)
    println(a)
    b = from_named(ref mut fixed)
    println(b)
    arena = Arena.with_capacity(64)
    scope = arena.scope()
    c = from_scoped(ref mut scope)
    println(c)
