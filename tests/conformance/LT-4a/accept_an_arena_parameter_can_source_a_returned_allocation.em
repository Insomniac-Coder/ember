#$ test: run-pass
#$ rules: LT-4, LT-4a, ARN-1
# Arena is not a view type. The narrow @borrows exception records that the
# returned mutable reference points into storage owned by this parameter.

@borrows(arena)
fn allocate_one(arena: Arena, value: i32) -> ref mut i32:
    return arena.alloc(value)

fn main():
    arena = Arena.with_capacity(16)
    stored = allocate_one(arena, 41)
    another = arena.alloc(42)
    println(stored)
    println(another)
#$ stdout: 41
#$ 42
