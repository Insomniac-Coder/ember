#$ test: compile-fail
#$ rules: ARN-3, TYP-16
# Opaque T is checked again after substitution; a concrete owning Array must
# not bypass E3090 merely because the generic definition could not decide.

fn put[T](arena: Arena, owned value: T):
    saved = arena.alloc(value) #$ error[E3090]: `Array[i32]` needs `drop` and cannot be allocated with `Arena.alloc`

fn main():
    arena = Arena.with_capacity(16)
    values: Array[i32] = Array[i32]()
    values.push(1)
    put(arena, values)
