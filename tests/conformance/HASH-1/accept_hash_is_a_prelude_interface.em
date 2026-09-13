#$ test: compile-pass
#$ rules: HASH-1, MOD-5

from std.collections import Hasher, DefaultHasher

struct Key:
    value: i32

extend Key implements Hash:
    fn hash[H: Hasher](self, mut h: H):
        h.write_i32(self.value)

fn accepts_hash[T: Hash](value: T) -> u64:
    hasher = DefaultHasher.new()
    value.hash(hasher)
    return hasher.finish()

fn main():
    println(accepts_hash(Key(1)))
