#$ test: compile-fail
#$ rules: EFF-20, EFF-5
#$ error[E4041]: `@noio` function `report` reaches I/O
# `[EFF-20]` — `@noio` forbids `Io`, including console output: `println` in a `@noio` function is
# `E4041`.

@noio
fn report(n: int):
    println(n)

fn main():
    report(1)
