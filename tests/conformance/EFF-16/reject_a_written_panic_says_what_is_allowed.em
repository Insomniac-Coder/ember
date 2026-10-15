#$ test: compile-fail
#$ rules: EFF-16, EFF-5
#$ error[E4040]: arithmetic, bounds, aliasing and stale-handle checks are allowed under it
# `[EFF-16]` — the diagnostic for `@nopanic(explicit)` says which panics it forbids and which
# checks it allows.

@nopanic(explicit)
fn f(n: int) -> int:
    assert(n > 0)
    return n

fn main():
    println(f(1))
