#$ test: run-pass
#$ rules: ARN-5a, ARN-5b, ARN-5g, TST-24

from std.collections import ArenaArray, ArenaMap, CapacityError

fn array_full(value: Result[void, CapacityError]) -> bool:
    match value:
        Ok(_) => return false
        Err(Full) => return true

fn map_full(value: Result[Option[i32], CapacityError]) -> bool:
    match value:
        Ok(_) => return false
        Err(Full) => return true

fn main():
    arena = Arena.with_capacity(1024)
    values: ArenaArray[i32] = ArenaArray[i32].with_capacity(arena, 0)
    println(values.capacity())
    println(array_full(values.push(1)))
    println(values.len())

    pairs: ArenaMap[i32, i32] = ArenaMap[i32, i32].with_capacity(arena, 0)
    println(pairs.capacity())
    println(map_full(pairs.insert(1, 2)))
    println(pairs.len())
#$ stdout: 0
#$ true
#$ 0
#$ 0
#$ true
#$ 0
