#$ test: run-pass
#$ rules: FFI-8
#$ profiles: debug, development, release
#$ stdout: 7 4 2 8 1.5
#$ stdout: true true true true
# `[FFI-8]` (D-742) — `std.ffi` names C's types: `c_int` is `i32`, `c_short` `i16`, `c_double`
# `f64`, and `import c` writes `int` as `c_int`. Those that differ between targets are the target's
# the compiler builds for, as this target's C compiler has them: `c_long`'s size, `c_char`'s and
# `c_wchar`'s signedness and `c_wchar`'s size. A typedef of one (`score_t`) is an alias of the alias,
# whichever order the program's modules are in (the program imports `c_int` itself, too).

from std.ffi import c_char, c_double, c_int, c_long, c_short, c_wchar
from std.mem import size_of
import c "c_types.h"

unsafe extern "C":
    @ffi(effects=[])
    safe fn abs(x: c_int) -> c_int

fn main():
    sum: c.score_t = 0
    long_size: usize = 0
    char_signed = false
    wchar_size: usize = 0
    wchar_signed = false
    unsafe:  # SAFETY(ffi): each function takes and returns scalars
        sum = c.add_ints(abs(-3), 4)
        long_size = c.long_size()
        char_signed = (c.char_min() as i32) == c.char_min_int()
        wchar_size = c.wchar_size()
        wchar_signed = ((c.wchar_minus_one() as i64) < 0) == (c.wchar_is_signed() == 1)
    half: c_double = 0.75
    println(sum, size_of[c_int](), size_of[c_short](), size_of[c_double](), half * 2.0)
    println(size_of[c_long]() == long_size, char_signed, size_of[c_wchar]() == wchar_size, wchar_signed)
