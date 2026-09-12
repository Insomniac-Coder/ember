#$ test: run-pass
#$ rules: TST-22, LT-4a

@borrows(arena)
fn allocate_one(arena: Arena) -> ref mut i32:
    return arena.alloc(9)

fn main():
    arena = Arena.with_capacity(16)
    value = allocate_one(arena)
    println(value)
#$ stdout: 9
