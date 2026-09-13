#$ test: compile-fail
#$ rules: TYP-16, TYP-18, HASH-4, ARN-5d, TYP-15a

from std.collections import ArenaMap, Hasher

@view
struct BorrowedKey:
    value: ref i32

extend BorrowedKey implements Eq:
    fn eq(self, other: Self) -> bool:
        return true

extend BorrowedKey implements Hash:
    fn hash[H: Hasher](self, mut h: H):
        h.write_u8(0)

@borrows(arena)
fn reserve[K: Eq + Hash, V](arena: Arena, capacity: usize) -> ArenaMap[K, V]:
    return ArenaMap[K, V].with_capacity(arena, capacity) #$ error[E2130]: an ArenaMap cannot store a view key or value

fn main():
    arena = Arena.with_capacity(1024)
    _values: ArenaMap[BorrowedKey, i32] = reserve[BorrowedKey, i32](arena, 1)
