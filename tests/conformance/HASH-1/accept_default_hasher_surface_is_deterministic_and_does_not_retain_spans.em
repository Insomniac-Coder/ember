#$ test: run-pass
#$ rules: HASH-1, HASH-2, HASH-3, BRW-2

from std.collections import DefaultHasher

fn feed(mut hasher: DefaultHasher, bytes: Span[u8]):
    hasher.write_bytes(bytes)
    hasher.write_u8(1)
    hasher.write_u16(2)
    hasher.write_u32(3)
    hasher.write_u64(4)
    hasher.write_i8(-1)
    hasher.write_i16(-2)
    hasher.write_i32(-3)
    hasher.write_i64(-4)
    hasher.write_usize(5)
    hasher.write_isize(-5)

fn main():
    bytes: [u8; 3] = [6, 7, 8]
    first = DefaultHasher.new()
    feed(first, bytes)
    first_hash = first.finish()

    second = DefaultHasher.new()
    feed(second, bytes)
    second_hash = second.finish()

    bytes[0] = 9
    println(first_hash == second_hash)
    println(bytes[0])
#$ stdout: true
#$ 9
