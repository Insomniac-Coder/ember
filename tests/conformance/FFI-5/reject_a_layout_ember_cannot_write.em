#$ test: compile-fail
#$ rules: FFI-5
#$ error[E5001]: the layout of `Header` does not match `packed_two.h`'s: its size is 8 bytes in Ember and 6 in C; its alignment is 4 in Ember and 2 in C; field `size` is at offset 4 in Ember and 2 in C
# `[FFI-5]` — layouts are verified, not assumed: a struct packed to two bytes (`#pragma pack(2)`)
# has no Ember spelling, and Ember's layout of it differs from Clang's. `E5001` names the type,
# each field and both numbers.

import c "packed_two.h"

fn main():
    pass
