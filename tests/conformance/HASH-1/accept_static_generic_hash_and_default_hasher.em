#$ test: run-pass
#$ rules: HASH-1, HASH-2, HASH-3

from std.collections import Hash, Hasher, DefaultHasher

struct Key:
    value: i32

struct CountingHasher implements Hasher:
    state: u64

    static fn new() -> CountingHasher:
        return CountingHasher(0)

    fn write_bytes(bytes: Span[u8]):
        index = 0
        while index < bytes.len():
            this.state = this.state ^ (bytes[index] as u64)
            index = index + 1

    fn write_u8(x: u8):
        this.state = this.state ^ (x as u64)

    fn write_u16(x: u16):
        this.state = this.state ^ (x as u64)

    fn write_u32(x: u32):
        this.state = this.state ^ (x as u64)

    fn write_u64(x: u64):
        this.state = this.state ^ x

    fn write_i8(x: i8):
        this.state = this.state ^ (x as u64)

    fn write_i16(x: i16):
        this.state = this.state ^ (x as u64)

    fn write_i32(x: i32):
        this.state = this.state ^ (x as u64)

    fn write_i64(x: i64):
        this.state = this.state ^ (x as u64)

    fn write_usize(x: usize):
        this.state = this.state ^ (x as u64)

    fn write_isize(x: isize):
        this.state = this.state ^ (x as u64)

    fn finish() owns this -> u64:
        return this.state

extend Key implements Hash:
    [H: needs Hasher]
    fn hash(ref mut h: H):
        h.write_i32(this.value)

[T: needs Hash]
fn hash_value(value: T) -> u64:
    hasher = DefaultHasher.new()
    value.hash(hasher)
    return hasher.finish()

[T: needs Hash, H: needs Hasher]
fn hash_with(value: T, owns hasher: H) -> u64:
    value.hash(hasher)
    return hasher.finish()

fn main():
    println(hash_value(7) == hash_value(7))
    println(hash_value(Key(7)) == hash_value(Key(7)))
    println(hash_value(Key(7)) == hash_value(Key(8)))
    println(hash_with(Key(7), CountingHasher.new()))
#$ stdout: true
#$ true
#$ false
#$ 7
