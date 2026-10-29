#$ test: run-pass
#$ rules: FFI-6
#$ warning[L3015]: `puts` is a C function the program calls in unsafe code
#$ warning[L3015]: `strlen` is a C function the program calls in unsafe code
#$ profiles: debug, development, release
#$ stdout: hi
#$ 2 2 true
# `[FFI-6]` (D-743) — a header may declare a function the C standard headers declare too: the
# generated C includes `<stdio.h>` and `<string.h>`, and Ember's own prototype of `puts` takes a
# `*c_char`, which C spells otherwise. Each foreign declaration binds its symbol under a C name of
# its own, so no two prototypes of one function meet: neither the import's, nor a hand
# declaration's, nor the system header's.

from std.ffi import c_char
import c "own_stdio.h"

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @safety("`text` is a NUL-terminated string")
    fn strlen(text: *c_char) -> usize

fn main():
    text: Array[c_char] = [104, 105, 0]
    pointer = text.as_span().as_ptr()
    unsafe:  # SAFETY(ffi): `text` is alive and NUL-terminated for the three calls
        written = c.puts(pointer)
        println(c.strlen(pointer), strlen(pointer), written >= 0)
