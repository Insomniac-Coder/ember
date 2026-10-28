#$ test: compile-fail
#$ rules: FFI-2
#$ error[E5002]: `safe fn` needs an `@ffi` contract for pointer parameter `items`
# `[FFI-2]` — a call is safe only when its contract has no unknown fact. A function with no overlay
# entry stays unsafe to call; an entry that leaves a pointer without facts asserts something it
# cannot (`E5002` at the entry's declaration); a `TODO(…)` word keeps the call unsafe (`[FFI-11c]`).

import c "./widgets.c" with (overlay = "overlays/partial.em") as w

fn main():
    items: [i32; 3] = [1, 2, 3]
    println(w.widget_add(1, 2))
    println(w.widget_freed())  #$ error[E3100]: `w.widget_freed()` (line 13) calls the C function `widget_freed`, whose contract is not verified
    w.widget_fill(ref_to_ptr(ref mut items[0]), 3, 1)  #$ error[E3100]: `w.widget_fill(ref_to_ptr(ref mut items[0]), 3, 1)` (line 14) calls the C function `widget_fill`, whose contract is not verified
