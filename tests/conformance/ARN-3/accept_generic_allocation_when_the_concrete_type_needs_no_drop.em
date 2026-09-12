#$ test: run-pass
#$ rules: ARN-3, TYP-16
# The needs_drop verdict is made for the concrete monomorphisation.

@borrows(arena)
fn put[T](arena: Arena, owned value: T) -> ref mut T:
    return arena.alloc(value)

fn main():
    arena = Arena.with_capacity(16)
    value = put(arena, 12)
    println(value)
#$ stdout: 12
