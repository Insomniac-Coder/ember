#$ test: run-pass
#$ rules: ARN-3, ARN-11, ARN-12, TST-23
# `[ARN-3]` — `alloc_array` initialises each element with `T.default()`
# (SP-017, ODR-049). A scalar's standard `Default` is its zero, so the
# array starts at zero whichever way it is filled; `alloc_zeroed` gives the
# same zero bytes by asking for them. Until SP-017 this case was "zeroed
# before `Default`", which no scalar could tell apart.

fn main():
    arena = Arena.with_capacity(16)
    values = arena.alloc_array[i32](1)
    zeroed = arena.alloc_zeroed[i32](1)
    println(values[0], zeroed[0])
#$ stdout: 0 0
