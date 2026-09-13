#$ test: run-pass
#$ rules: ARN-5d, ARN-5e, HASH-4, OWN-3, TST-24

from std.collections import ArenaMap, CapacityError, Hasher

struct Key:
    value: i32

struct Payload:
    value: i32

extend Key implements Eq:
    fn eq(self, other: Self) -> bool:
        return self.value == other.value

extend Key implements Hash:
    fn hash[H: Hasher](self, mut h: H):
        h.write_i32(self.value)

fn replaced(value: Result[Option[Payload], CapacityError]) -> i32:
    match value:
        Ok(previous):
            match previous:
                Some(found):
                    return found.value
                None:
                    return 0
        Err(_full):
            return -1

fn removed(value: Option[Payload]) -> i32:
    match value:
        Some(found):
            return found.value
        None:
            return -1

fn viewed(value: Option[ref Payload]) -> i32:
    match value:
        Some(found):
            return found.value
        None:
            return -1

fn main():
    arena = Arena.with_capacity(4096)
    values: ArenaMap[Key, Payload] = ArenaMap[Key, Payload].with_capacity(arena, 2)
    println(replaced(values.insert(Key(1), Payload(10))))
    println(replaced(values.insert(Key(2), Payload(20))))
    println(replaced(values.insert(Key(1), Payload(11))))
    println(removed(values.remove(Key(1))))
    println(viewed(values.get(Key(2))))
#$ stdout: 0
#$ 0
#$ 10
#$ 11
#$ 20
