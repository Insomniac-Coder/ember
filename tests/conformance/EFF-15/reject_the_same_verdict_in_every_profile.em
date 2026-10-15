#$ test: compile-fail
#$ rules: EFF-15, PHIL-13
#$ profiles: debug, release, shipping
#$ error[E4001]: `@noalloc` function `grow` reaches an allocation
# `[EFF-15]` — effects and contract verdicts are computed once and are the same in every profile
# (`[PHIL-13]`): no profile accepts this.

@noalloc
fn grow(mut xs: Array[int], n: int):
    if n > 0:
        xs.push(n)

fn main():
    xs: Array[int] = []
    grow(xs, 1)
    println(xs.len())
