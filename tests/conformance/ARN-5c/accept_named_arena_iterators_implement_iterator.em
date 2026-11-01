#$ test: run-pass
#$ rules: ARN-5c, ARN-5d, TST-24, TYP-17, IFC-4

from std.core import Iterator
from std.collections import ArenaArray, ArenaMap

[I: needs Iterator]
fn has_next(ref mut iterator: I) -> bool:
    match iterator.next():
        Some(_) => return true
        None => return false

fn main():
    arena = Arena.with_capacity(4096)

    values: ArenaArray[i32] = ArenaArray[i32].with_capacity(arena, 1)
    _pushed = values.push(7)
    array_iter = values.iter()
    println(has_next(ref mut array_iter))

    mut_iter = values.iter_mut()
    println(has_next(ref mut mut_iter))

    pairs: ArenaMap[i32, i32] = ArenaMap[i32, i32].with_capacity(arena, 1)
    _inserted = pairs.insert(4, 9)
    map_iter = pairs.iter()
    println(has_next(ref mut map_iter))
#$ stdout: true
#$ true
#$ true
