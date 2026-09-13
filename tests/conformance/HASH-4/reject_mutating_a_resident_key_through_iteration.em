#$ test: compile-fail
#$ rules: HASH-4, ARN-5d, BRW-1

from std.collections import ArenaMap

fn main():
    arena = Arena.with_capacity(2048)
    values: ArenaMap[i32, i32] = ArenaMap[i32, i32].with_capacity(arena, 1)
    _inserted = values.insert(1, 10)
    for pair in values.iter():
        key: ref i32 = pair.0
        key = 2 #$ error[E3021]: cannot write through a shared reference
