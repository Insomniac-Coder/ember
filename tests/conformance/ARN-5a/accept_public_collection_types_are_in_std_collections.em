#$ test: compile-pass
#$ rules: ARN-5a

from std.collections import ArenaArray, ArenaMap, CapacityError, ArenaArrayIter, ArenaArrayIterMut, ArenaMapIter

fn names(
    _array: ArenaArray[i32],
    _map: ArenaMap[i32, i64],
    _error: CapacityError,
    _iter: ArenaArrayIter[i32],
    _iter_mut: ArenaArrayIterMut[i32],
    _map_iter: ArenaMapIter[i32, i64],
):
    pass

fn main():
    println(0)
