#$ test: compile-fail
#$ rules: ARN-2, ARN-3, ARN-11, TST-23

fn main():
    arena = Arena.with_capacity(64)
    values = arena.alloc_array[Array[i32]](2) #$ error[E3090]: `Array[i32]` needs `drop` and cannot be allocated with `Arena.alloc_array`
    println(values.len())
