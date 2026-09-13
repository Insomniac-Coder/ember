#$ test: run-pass
#$ rules: HASH-1, HASH-3, HASH-4, ARN-5d

from std.collections import ArenaMap, CapacityError, Hasher, DefaultHasher

struct Key:
    value: i32

extend Key implements Eq:
    fn eq(self, other: Self) -> bool:
        return self.value == other.value

extend Key implements Hash:
    fn hash[H: Hasher](self, mut h: H):
        h.write_i32(self.value)

@borrows(arena)
fn reserve_map[K: Eq + Hash, V](arena: Arena, capacity: usize) -> ArenaMap[K, V]:
    return ArenaMap[K, V].with_capacity(arena, capacity)

fn has_key[K: Eq + Hash, V](values: ArenaMap[K, V], key: K) -> bool:
    return values.contains_key(key)

fn hash_value(key: Key) -> u64:
    hasher = DefaultHasher.new()
    key.hash(hasher)
    return hasher.finish()

fn inserted(value: Result[Option[i32], CapacityError]) -> i32:
    match value:
        Ok(previous):
            match previous:
                Some(found):
                    return found
                None:
                    return 0
        Err(_full):
            return -1

fn viewed(value: Option[ref i32]) -> i32:
    match value:
        Some(found):
            return found
        None:
            return -1

fn removed(value: Option[i32]) -> i32:
    match value:
        Some(found):
            return found
        None:
            return -1

fn main():
    arena = Arena.with_capacity(4096)
    builtin_values: ArenaMap[i32, i32] = reserve_map[i32, i32](arena, 1)
    println(inserted(builtin_values.insert(4, 40)))
    values: ArenaMap[Key, i32] = reserve_map[Key, i32](arena, 2)
    println(hash_value(Key(3)) == hash_value(Key(3)))
    println(inserted(values.insert(Key(1), 10)))
    println(inserted(values.insert(Key(2), 20)))
    println(inserted(values.insert(Key(1), 11)))
    println(viewed(values.get(Key(1))))
    println(has_key(values, Key(2)))
    println(removed(values.remove(Key(1))))
    println(viewed(values.get(Key(2))))
    for pair in values.iter():
        println(pair.1)
#$ stdout: 0
#$ true
#$ 0
#$ 0
#$ 10
#$ 11
#$ true
#$ 11
#$ 20
#$ 20
