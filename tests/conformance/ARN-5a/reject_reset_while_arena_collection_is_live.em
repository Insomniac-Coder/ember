#$ test: compile-fail
#$ rules: ARN-1, ARN-5a, ARN-5f, LT-4a, TST-24

from std.collections import ArenaArray

fn main():
    arena = Arena.with_capacity(1024)
    values: ArenaArray[i32] = ArenaArray[i32].with_capacity(arena, 2)
    arena.reset() #$ error[E3021]: `arena` is borrowed here and mutably borrowed elsewhere
    println(values.len())
