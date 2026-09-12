#$ test: compile-fail
#$ rules: LT-4, LT-4a

@borrows(arena)
fn allocate_one(arena: Arena) -> ref mut i32:
    return arena.alloc(1)

fn escaped() -> ref mut i32:
    arena = Arena.with_capacity(16)
    return allocate_one(arena) #$ error[E3061]: arena allocation cannot outlive `arena`

fn main():
    println(0)
