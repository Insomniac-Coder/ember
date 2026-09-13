#$ test: run-pass
#$ rules: ARN-5b, ARN-5f, BRW-1, TST-24

from std.collections import ArenaArray

fn main():
    arena = Arena.with_capacity(1024)
    values: ArenaArray[i32] = ArenaArray[i32].with_capacity(arena, 2)
    _pushed = values.push(7)
    match values.get(0):
        Some(value) => println(value)
        None => println(-1)
    values.clear()
    println(values.len())
#$ stdout: 7
#$ 0
