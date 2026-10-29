#$ test: run-pass
#$ rules: EFF-2, EFF-1
#$ stdout: 4
#$ stdout: 2
# `[EFF-2]` (ODR 15.3) — dropping a function value has the effects of dropping the functions stored
# into its type: `scaled` owns its `Scaler` and drops it, and the only functions of `by`'s type are
# `double` and `widen`, named functions holding nothing, so the drop does nothing `@noalloc` forbids.

struct Scaler:
    by: fn(int) -> int

struct Logger:
    format: fn(int) -> int

fn double(x: int) -> int:
    return x * 2

fn widen(x: int) -> int:
    return [x, x].len()

@noalloc
fn scaled(owned s: Scaler, x: int) -> int:
    f = s.by
    return f(x)

fn main():
    times = Scaler(by = double)
    log = Logger(format = widen)
    println(scaled(times, 2))
    println(log.format(1))
