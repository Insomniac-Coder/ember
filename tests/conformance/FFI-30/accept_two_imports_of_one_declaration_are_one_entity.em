#$ test: run-pass
#$ rules: FFI-30, FFI-30a
#$ profiles: debug, development, release
#$ stdout: 25.0 5.0 2 true
#$ stdout: 2.0
# `[FFI-30]` — two imports of one C declaration with the same layout are one Ember entity, whichever
# module or alias imports them: `left.h` and `right.h` both include `common.h`, so a `Vec2` one
# makes is the other's, as is a `Mode` and a `struct Handle*`. `[FFI-30a]` — an overlay changes
# functions, never a type: `left.h` imported again with an overlay that makes `left_make` safe still
# makes the one `Vec2`.

import c "left.h" as left
import c "right.h" as right
import c "left.h" with (overlay = "overlays/left.em") as safe_left

fn main():
    w = right.Vec2(x=1.0, y=2.0)
    unsafe:  # SAFETY(ffi): scalar and by-value calls, and a null handle compared
        v = left.left_make(3.0, 4.0)
        println(right.right_length2(v), right.right_length2(w), right.right_mode_value(left.left_mode(2)), right.right_is_null(left.left_handle() as *right.Handle) == 1)
    u = safe_left.left_make(1.0, 1.0)
    unsafe:  # SAFETY(ffi): a by-value call
        println(right.right_length2(u))
