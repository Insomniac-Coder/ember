#$ test: run-pass
#$ rules: ARN-5a, LT-4a, TST-24, TYP-18

from std.collections import ArenaArray

@borrows(arena)
fn reserve[T](arena: Arena, capacity: usize) -> ArenaArray[T]:
    return ArenaArray[T].with_capacity(arena, capacity)

fn main():
    arena = Arena.with_capacity(1024)
    values: ArenaArray[i32] = reserve[i32](arena, 2)
    _pushed = values.push(9)
    println(values.len())
#$ stdout: 1
