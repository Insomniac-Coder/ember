#$ test: run-pass
#$ rules: EFF-2, EFF-5
#$ stdout: 6
#$ stdout: 2
# `[EFF-2]` (verdict 128) — a callable parameter holds what its callers pass: `apply`'s `f` is given
# only `s.by`, which holds the lambda that multiplies, so `scaled` keeps `@noalloc`; `Logger.format`'s
# lambda, which allocates, is of the same type and is never passed to `apply`.

struct Scaler:
    by: fn(int) -> int

struct Logger:
    format: fn(int) -> int

fn apply(mut f: fn(int) -> int, x: int) -> int:
    return f(x)

@noalloc
fn scaled(mut s: Scaler, x: int) -> int:
    return apply(s.by, x)

fn main():
    k = 3
    times = Scaler(by = owned fn(x: int) -> int => x * k)
    log = Logger(format = owned fn(x: int) -> int => [x, k].len())
    println(scaled(times, 2))
    println(log.format(1))
