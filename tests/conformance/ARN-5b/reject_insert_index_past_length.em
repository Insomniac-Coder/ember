#$ test: run-fail
#$ rules: ARN-5b, ARN-5c, TST-24
#$ panics: index 1 is out of bounds for a length of 0

from std.collections import ArenaArray

fn main():
    arena = Arena.with_capacity(1024)
    values: ArenaArray[i32] = ArenaArray[i32].with_capacity(arena, 2)
    _result = values.insert(1, 7)
