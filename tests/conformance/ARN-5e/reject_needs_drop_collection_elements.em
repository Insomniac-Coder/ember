#$ test: compile-fail
#$ rules: ARN-2, ARN-5e, TST-24

from std.collections import ArenaArray, ArenaMap

struct Bag:
    values: Array[i32]

fn main():
    arena = Arena.with_capacity(2048)
    _array: ArenaArray[Bag] = ArenaArray[Bag].with_capacity(arena, 1) #$ error[E3090]: `ArenaArray[Bag]` requires `!needs_drop(Bag)`
    _map: ArenaMap[i32, Bag] = ArenaMap[i32, Bag].with_capacity(arena, 1) #$ error[E3090]: `ArenaMap[i32, Bag]` requires drop-free keys and values
