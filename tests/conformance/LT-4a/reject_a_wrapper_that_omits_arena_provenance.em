#$ test: compile-fail
#$ rules: LT-4, LT-4a

fn allocate_one(arena: Arena) -> ref mut i32:
    return arena.alloc(1) #$ error[E3061]: arena allocation cannot outlive `arena`

fn main():
    println(0)
