#$ test: run-pass
#$ rules: FFI-8, FFI-5
#$ warning[L3015]: `flags_make` is a C function the program calls in unsafe code
#$ warning[L3015]: `flags_level` is a C function the program calls in unsafe code
#$ warning[L3015]: `flags_delta` is a C function the program calls in unsafe code
#$ warning[L3015]: `flags_wide` is a C function the program calls in unsafe code
#$ warning[L3015]: `flags_size` is a C function the program calls in unsafe code
#$ profiles: debug, development, release
#$ stdout: 7 1 9 -6 true 300 78187493530
#$ stdout: 5 -16 1099511627775
#$ stdout: 5 -16 false 1099511627775 1 7 300 true
# `[FFI-8]` — a bit-field is read and written by `get_<f>`/`set_<f>` methods, never addressed: the
# importer lays the struct out as this target's C compiler does (`[FFI-5]` checks it), the bytes
# bit-fields share as a byte array, and each accessor reads or writes only its field's bits — signed
# ones sign-extended, `bool` ones as `bool`, a 40-bit one past a zero-width field. C reads what Ember
# wrote, and the neighbours of a written field keep their values.

from std.ffi import c_int, c_uint
from std.mem import size_of
import c "bitfields.h"

fn main():
    f = c.Flags(tag = 0, count = 0)
    unsafe:  # SAFETY(ffi): the functions take and return values
        f = c.flags_make()
    println(f.tag, f.get_ready(), f.get_level(), f.get_delta(), f.get_on(), f.count, f.get_wide())
    g = f
    g.set_level(5)
    g.set_delta(-16)
    g.set_on(false)
    g.set_wide(1099511627775)
    level: c_uint = 0
    delta: c_int = 0
    wide: u64 = 0
    same_size = false
    unsafe:  # SAFETY(ffi): as above
        level = c.flags_level(g)
        delta = c.flags_delta(g)
        wide = c.flags_wide(g)
        same_size = size_of[c.Flags]() == c.flags_size()
    println(level, delta, wide)
    println(g.get_level(), g.get_delta(), g.get_on(), g.get_wide(), g.get_ready(), g.tag, g.count, same_size)
