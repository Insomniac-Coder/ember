#$ test: compile-fail
#$ rules: ARN-3, ARN-11, PHIL-10, TST-23

fn main():
    arena = Arena.with_capacity(16)
    values = arena.alloc_array[ref i32](1) #$ error[E2040]: `ref i32` has no available `Zeroable` or `Default` capability
    println(values.len())
