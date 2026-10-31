#$ test: compile-pass
#$ rules: HASH-1, MOD-5

from std.collections import Hasher, DefaultHasher

struct Key:
    value: i32

extend Key implements Hash:
    [H: needs Hasher]
    fn hash(ref mut h: H):
        h.write_i32(this.value)

[T: needs Hash]
fn accepts_hash(value: T) -> u64:
    hasher = DefaultHasher.new()
    value.hash(hasher)
    return hasher.finish()

fn main():
    println(accepts_hash(Key(1)))
