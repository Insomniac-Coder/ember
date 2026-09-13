#$ test: compile-fail
#$ rules: ARN-5f, BRW-1, TST-24

from std.collections import ArenaArray

fn main():
    arena = Arena.with_capacity(1024)
    values: ArenaArray[i32] = ArenaArray[i32].with_capacity(arena, 2)
    _pushed = values.push(7)
    iterator = values.iter()
    values.clear() #$ error[E3021]: `values` is borrowed here and mutably borrowed elsewhere
    match iterator.next():
        Some(value) => println(value)
        None => println(-1)
