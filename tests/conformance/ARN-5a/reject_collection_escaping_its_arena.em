#$ test: compile-fail
#$ rules: ARN-5, ARN-5a, LT-4a, TST-24

from std.collections import ArenaArray

fn invalid() -> ArenaArray[i32]:
    arena = Arena.with_capacity(1024)
    return ArenaArray[i32].with_capacity(arena, 1) #$ error[E3061]: arena allocation cannot outlive `arena`

fn main():
    println(0)
