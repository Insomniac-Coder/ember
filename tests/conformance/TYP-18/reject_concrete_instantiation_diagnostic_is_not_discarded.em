#$ test: compile-fail
#$ rules: TYP-16, TYP-18, HASH-4, ARN-5d, TYP-15a

from std.collections import ArenaMap, Hasher

@view
struct BorrowedKey:
    value: ref i32

extend BorrowedKey implements Eq:
    fn eq(other: Self) -> bool:
        return true

extend BorrowedKey implements Hash:
    [H: needs Hasher]
    fn hash(ref mut h: H):
        h.write_u8(0)

[K: needs (Eq, Hash), V]
fn reserve(arena: Arena, capacity: usize) -> ArenaMap[K, V] from arena:
    return ArenaMap[K, V].with_capacity(arena, capacity) #$ error[E0900]: an `ArenaMap` with a view key or value is not built by this compiler yet

fn main():
    arena = Arena.with_capacity(1024)
    _values: ArenaMap[BorrowedKey, i32] = reserve[BorrowedKey, i32](arena, 1)
