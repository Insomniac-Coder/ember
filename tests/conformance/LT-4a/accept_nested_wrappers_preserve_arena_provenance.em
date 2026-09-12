#$ test: run-pass
#$ rules: LT-4a, LT-4, LT-1a

@borrows(arena)
fn inner(arena: Arena) -> ref mut i32:
    return arena.alloc(7)

@borrows(arena)
fn outer(arena: Arena) -> ref mut i32:
    return inner(arena)

fn main():
    arena = Arena.with_capacity(16)
    value = outer(arena)
    println(value)
#$ stdout: 7
