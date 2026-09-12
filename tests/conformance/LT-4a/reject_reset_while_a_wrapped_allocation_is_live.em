#$ test: compile-fail
#$ rules: LT-4, LT-4a, BRW-1

@borrows(arena)
fn allocate_one(arena: Arena) -> ref mut i32:
    return arena.alloc(1)

fn main():
    arena = Arena.with_capacity(16)
    value = allocate_one(arena)
    arena.reset() #$ error[E3021]: `arena` is borrowed here and mutably borrowed elsewhere
    println(value)
