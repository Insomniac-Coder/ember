#$ test: run-pass
#$ rules: FFI-6a, FFI-8
#$ profiles: debug, development, release
#$ stdout: 12 25 255 1099511627776 -7 65 8
#$ stdout: 0.25 0.5 hi	there concat
#$ stdout: 4 4 8 8 4 4 8
#$ stdout: 4294967295 -1 18446744073709551615
# `[FFI-6a]` — an object-like macro that Clang evaluates to an integer, floating or string constant
# is a constant of the C type of that value: `int` is `i32`, `0xFFu` `u32`, `1ULL << 40` `u64`,
# `'A'` an `int`, `0.5f` `f32`; `sizeof` is `size_t`, `usize`. `[FFI-8]` — `#define S "text"` is
# `const S: cstr = c"text"`.

import c "constants.h"
from std.mem import size_of

# The width of the type a value was given.
fn width[T](value: T) -> usize:
    return size_of[T]()

fn main():
    println(c.CONSTANTS_COUNT, c.CONSTANTS_DOUBLED, c.CONSTANTS_MASK, c.CONSTANTS_BIG, c.CONSTANTS_NEGATIVE, c.CONSTANTS_LETTER, c.CONSTANTS_SIZE)
    println(c.CONSTANTS_RATIO, c.CONSTANTS_HALF, c.CONSTANTS_GREETING.to_str().unwrap(), c.CONSTANTS_JOINED.to_str().unwrap())
    # Each constant has its C type: its width, and whether it is signed.
    println(width(c.CONSTANTS_COUNT), width(c.CONSTANTS_MASK), width(c.CONSTANTS_BIG), width(c.CONSTANTS_RATIO), width(c.CONSTANTS_HALF), width(c.CONSTANTS_LETTER), width(c.CONSTANTS_SIZE))
    println(c.CONSTANTS_MASK.wrapping_sub(256), c.CONSTANTS_COUNT.wrapping_sub(13), c.CONSTANTS_SIZE.wrapping_sub(9))
