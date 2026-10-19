#$ test: run-pass
#$ rules: FFI-6a, FFI-8
#$ profiles: debug, development, release
#$ stdout: 12 25 255 1099511627776 -7 65 8
#$ stdout: 0.25 0.5 hi	there concat
#$ stdout: 12 255 1099511627776 0.25 0.5 8 65
# `[FFI-6a]` — an object-like macro that Clang evaluates to an integer, floating or string constant
# is a constant of the C type of that value: `int` is `i32`, `0xFFu` `u32`, `1ULL << 40` `u64`,
# `'A'` an `int`, `0.5f` `f32`; `sizeof` is `size_t`, `usize`. `[FFI-8]` — `#define S "text"` is
# `const S: cstr = c"text"`.

import c "constants.h"

fn main():
    println(c.CONSTANTS_COUNT, c.CONSTANTS_DOUBLED, c.CONSTANTS_MASK, c.CONSTANTS_BIG, c.CONSTANTS_NEGATIVE, c.CONSTANTS_LETTER, c.CONSTANTS_SIZE)
    println(c.CONSTANTS_RATIO, c.CONSTANTS_HALF, c.CONSTANTS_GREETING.to_str().unwrap(), c.CONSTANTS_JOINED.to_str().unwrap())
    # Each constant has its C type: a binding of another type would be `E2020`.
    count: i32 = c.CONSTANTS_COUNT
    mask: u32 = c.CONSTANTS_MASK
    big: u64 = c.CONSTANTS_BIG
    ratio: f64 = c.CONSTANTS_RATIO
    half: f32 = c.CONSTANTS_HALF
    size: usize = c.CONSTANTS_SIZE
    letter: i32 = c.CONSTANTS_LETTER
    println(count, mask, big, ratio, half, size, letter)
