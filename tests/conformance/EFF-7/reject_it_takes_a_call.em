#$ test: compile-fail
#$ rules: EFF-7
#$ error[E0104]: `@assume_noalloc` takes a call
# `[EFF-7]` — the override is for one call the compiler cannot see into; anything else is
# refused.

@noalloc
fn twice(x: int) -> int:
    n: int = 0
    unsafe:  # SAFETY: a test of what the override admits
        n = @assume_noalloc(x + x)
    return n

fn main():
    println(twice(2))
