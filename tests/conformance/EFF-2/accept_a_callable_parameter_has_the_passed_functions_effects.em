#$ test: run-pass
#$ rules: EFF-2, DET-3
#$ stdout: 4
# `[EFF-2]` — a call through a callable parameter contributes the effects of the function actually
# passed, per instantiation: `double` allocates nothing, so `@noalloc` holds.

fn apply(f: fn(int) -> int, x: int) -> int:
    return f(x)

fn double(x: int) -> int:
    return x * 2

@noalloc
@deterministic
fn uses(x: int) -> int:
    return apply(double, x)

fn main():
    println(uses(2))
