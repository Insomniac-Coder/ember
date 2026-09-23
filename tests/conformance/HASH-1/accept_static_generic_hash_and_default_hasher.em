#$ test: run-pass
#$ rules: HASH-1, HASH-2, HASH-3

from std.collections import Hash, Hasher, DefaultHasher

struct Key:
    value: i32

struct CountingHasher implements Hasher:
    state: u64

    fn new() -> CountingHasher:
        return CountingHasher(0)

    fn write_bytes(mut self, bytes: Span[u8]):
        index = 0
        while index < bytes.len():
            self.state = self.state ^ (bytes[index] as u64)
            index = index + 1

    fn write_u8(mut self, x: u8):
        self.state = self.state ^ (x as u64)

    fn write_u16(mut self, x: u16):
        self.state = self.state ^ (x as u64)

    fn write_u32(mut self, x: u32):
        self.state = self.state ^ (x as u64)

    fn write_u64(mut self, x: u64):
        self.state = self.state ^ x

    fn write_i8(mut self, x: i8):
        self.state = self.state ^ (x as u64)

    fn write_i16(mut self, x: i16):
        self.state = self.state ^ (x as u64)

    fn write_i32(mut self, x: i32):
        self.state = self.state ^ (x as u64)

    fn write_i64(mut self, x: i64):
        self.state = self.state ^ (x as u64)

    fn write_usize(mut self, x: usize):
        self.state = self.state ^ (x as u64)

    fn write_isize(mut self, x: isize):
        self.state = self.state ^ (x as u64)

    fn finish(owned self) -> u64:
        return self.state

extend Key implements Hash:
    fn hash[H: Hasher](self, mut h: H):
        h.write_i32(self.value)

fn hash_value[T: Hash](value: T) -> u64:
    hasher = DefaultHasher.new()
    value.hash(hasher)
    return hasher.finish()

fn hash_with[T: Hash, H: Hasher](value: T, owned hasher: H) -> u64:
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
