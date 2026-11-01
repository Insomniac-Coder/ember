#$ test: compile-fail
#$ rules: STR-1, ARN-1
# `[STR-1]` — the memberwise constructor is `pub` only when every field is. A compiler-known struct's
# fields are all its own, so `Arena()` is refused once, and its fields are not named (an arena is
# made by `Arena.with_capacity(n)`, `[ARN-1]`).

fn main():
    arena = Arena()    #$ error[E1052]: `Arena`'s memberwise constructor is private to its module
    fixed = FixedArena()    #$ error[E1052]: `FixedArena`'s memberwise constructor is private to its module
    println(1)
