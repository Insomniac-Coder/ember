#$ test: compile-fail
#$ rules: EFF-2
#$ error[E4001]: `@noalloc` function `uses` reaches an allocation
# `[EFF-2]` — the same `apply` given a function that allocates carries its allocation.

fn apply(f: fn(int) -> int, x: int) -> int:
    return f(x)

fn grow(x: int) -> int:
    xs = [x, x]
    return xs.len()

@noalloc
fn uses(x: int) -> int:
    return apply(grow, x)

fn main():
    println(uses(2))
