#$ test: run-pass
#$ rules: EFF-2, EFF-5, CLO-10
#$ profiles: debug, development, release
#$ stdout: 6
#$ stdout: 2
# `[EFF-2]` (verdict 122) — a call through a field of a callable type runs one of the functions that
# reach that field: each field of each type apart. `Logger.format` holds a lambda that allocates, and
# `Scaler.by` only one that multiplies, so the `@noalloc` function calling through `Scaler.by` is
# accepted. (By type, both lambdas are a `fn(int) -> int`, and it was refused.)

struct Scaler:
    by: fn(int) -> int

struct Logger:
    format: fn(int) -> int

@noalloc
fn scaled(mut s: Scaler, x: int) -> int:
    return s.by(x)

fn main():
    k = 3
    times = Scaler(by = owned fn(x: int) -> int => x * k)
    log = Logger(format = owned fn(x: int) -> int => [x, k].len())
    println(scaled(times, 2))
    println(log.format(1))
