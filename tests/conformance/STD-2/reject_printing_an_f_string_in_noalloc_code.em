#$ test: compile-fail
#$ rules: STD-2
#$ error[E4001]: `@noalloc` function `report` reaches an allocation
# `[STD-2]` — printing allocates to build an f-string argument.

@noalloc
fn report(n: int):
    println(f"n = {n}")

fn main():
    report(3)
