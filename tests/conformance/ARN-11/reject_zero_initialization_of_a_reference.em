#$ test: compile-fail
#$ rules: ARN-3, ARN-11, PHIL-10, TST-23
# `[ARN-11]` — a `ref` is never `Zeroable`, so zero bytes cannot make one;
# and it has no `Default` for `alloc_array` to call (SP-017, ODR-049).

fn main():
    arena = Arena.with_capacity(16)
    zeroed = arena.alloc_zeroed[ref i32](1) #$ error[E2040]: `ref i32` is not `Zeroable`: nothing shows that all-zero bytes are a valid `ref i32`
    values = arena.alloc_array[ref i32](1) #$ error[E2040]: `ref i32` does not implement `Default`, which `alloc_array` needs
    println(zeroed.len(), values.len())
