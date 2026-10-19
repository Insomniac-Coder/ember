#$ test: run-pass
#$ rules: LAY-2
#$ profiles: debug, development, release
#$ stdout: 5 7 7 | 16 8 9
# `[LAY-2]` — `size_of[T]()`, `align_of[T]()` and `offset_of[T](field)` are compile-time
# functions: each is a constant wherever a constant goes (a `const`, an array's length), and in
# generic code each instance computes its own. `offset_of` is `std.mem`'s, also by its path.
import std.mem

@packed
struct Header:
    tag: u8
    length: u32
    flags: u16

struct Pair[T]:
    a: u8
    b: T

const FLAGS_AT: usize = offset_of[Header](flags)
const HEADER_BYTES: usize = size_of[Header]() + align_of[Header]() - 1

fn second_at[T]() -> usize:
    return offset_of[Pair[T]](b)

fn main():
    buffer: [u8; HEADER_BYTES] = [0; 7]
    flags: [u8; mem.offset_of[Header](flags) + 2] = [1; 7]
    println(FLAGS_AT, len(buffer) + buffer[0] as int, len(flags) + flags[0] as int - 1, "|", second_at[u128](), second_at[f64](), offset_of[Pair[u64]](b) + 1)
