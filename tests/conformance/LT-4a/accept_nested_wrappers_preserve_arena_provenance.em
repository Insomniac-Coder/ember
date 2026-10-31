#$ test: run-pass
#$ rules: LT-4a, LT-4, LT-1a

fn inner(arena: Arena) -> ref mut i32 from arena:
    return arena.alloc(7)

fn outer(arena: Arena) -> ref mut i32 from arena:
    return inner(arena)

fn main():
    arena = Arena.with_capacity(16)
    value = outer(arena)
    println(value)
#$ stdout: 7
