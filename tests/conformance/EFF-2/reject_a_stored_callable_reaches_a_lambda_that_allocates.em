#$ test: compile-fail
#$ rules: EFF-2, EFF-5, CLO-10
# `[EFF-2]` (A4-1) — a call through a field of a callable type has the effects of every function the
# program stores into that type: one of the two lambdas allocates, so the `@noalloc` function calling
# through the field reaches that allocation, whichever lambda the field holds.

struct Scaler:
    by: fn(int) -> int

@noalloc
fn scaled(mut s: Scaler, x: int) -> int:
    return s.by(x)    #$ error[E4001]: `@noalloc` function `scaled` reaches an allocation

fn main():
    k = 3
    times = Scaler(by = owned fn(x: int) -> int => x * k)
    widths = Scaler(by = owned fn(x: int) -> int => [x, k].len())
    println(scaled(times, 2), scaled(widths, 2))
