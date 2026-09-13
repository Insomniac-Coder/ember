#$ test: compile-fail
#$ rules: HASH-4, ARN-5d

from std.collections import ArenaMap

struct EqOnly:
    value: i32

extend EqOnly implements Eq:
    fn eq(self, other: Self) -> bool:
        return self.value == other.value

fn main():
    arena = Arena.with_capacity(1024)
    _values: ArenaMap[EqOnly, i32] = ArenaMap[EqOnly, i32].with_capacity(arena, 1) #$ error[E2040]: `EqOnly` does not implement `Hash`, which ArenaMap keys require
