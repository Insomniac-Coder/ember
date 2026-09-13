#$ test: compile-fail
#$ rules: HASH-4, ARN-5d

from std.collections import ArenaMap, Hasher

struct HashOnly:
    value: i32

extend HashOnly implements Hash:
    fn hash[H: Hasher](self, mut h: H):
        h.write_i32(self.value)

fn main():
    arena = Arena.with_capacity(1024)
    _values: ArenaMap[HashOnly, i32] = ArenaMap[HashOnly, i32].with_capacity(arena, 1) #$ error[E2040]: `HashOnly` does not implement `Eq`, which ArenaMap keys require
