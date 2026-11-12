#$ test: compile-fail
#$ rules: OWN-6
# `[OWN-6]` — `std.mem` has `take`, `replace`, `swap` and `drop`, and no `forget`: importing it is
# `E1010`, as calling it is (`reject_mem_forget_is_gone.em`).

from std.mem import drop, forget    #$ error[E1010]: `mem.forget` is gone: a value always ends with its clean-up

fn main():
    value = 5
    drop(value)
