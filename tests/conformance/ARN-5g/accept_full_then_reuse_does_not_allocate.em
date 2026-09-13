#$ test: run-pass
#$ rules: ARN-5g, TST-24
#$ assert-c-count: contains("ember_arena_alloc_uninit") == 1

from std.collections import ArenaArray

fn main():
    arena = Arena.with_capacity(1024)
    values: ArenaArray[i32] = ArenaArray[i32].with_capacity(arena, 1)
    _first = values.push(1)
    _full = values.push(2)
    values.clear()
    _again = values.push(3)
    println(values.len())
#$ stdout: 1
