#$ test: run-pass
#$ rules: ARN-5, ARN-5g, TST-24
#$ assert-c-count: contains("ember_arena_alloc_uninit") == 1
#$ assert-c-count: contains("ember_arena_alloc_zeroed") == 1

from std.collections import ArenaArray, ArenaMap

fn main():
    arena = Arena.with_capacity(4096)
    values: ArenaArray[i32] = ArenaArray[i32].with_capacity(arena, 2)
    _push = values.push(1)
    _insert = values.insert(0, 2)
    _remove = values.remove(1)
    values.clear()

    pairs: ArenaMap[i32, i32] = ArenaMap[i32, i32].with_capacity(arena, 2)
    _new = pairs.insert(1, 2)
    _replace = pairs.insert(1, 3)
    _removed = pairs.remove(1)
    pairs.clear()
    println(values.len() + pairs.len())
#$ stdout: 0
