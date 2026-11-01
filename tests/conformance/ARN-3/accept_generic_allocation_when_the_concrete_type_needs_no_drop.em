#$ test: run-pass
#$ rules: ARN-3, TYP-16
# The needs_drop verdict is made for the concrete monomorphisation.

[T]
fn put(arena: Arena, owns value: T) -> ref mut T from arena:
    return arena.alloc(value)

fn main():
    arena = Arena.with_capacity(16)
    value = put(arena, owns 12)
    println(value)
#$ stdout: 12
