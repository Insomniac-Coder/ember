#$ test: run-pass
#$ rules: EFF-2, EFF-5
#$ stdout: 2
#$ stdout: 2
# `[EFF-2]` (verdict 129) — each list holds what is put into it, and no other list's: `widen`, which
# allocates, is in `Log.lines` only, so the `@noalloc` function calling through `Bus.handlers`, which
# holds `double` and `triple`, is accepted.

struct Bus:
    handlers: Array[fn(int) -> int]

struct Log:
    lines: Array[fn(int) -> int]

fn double(x: int) -> int:
    return x * 2

fn triple(x: int) -> int:
    return x * 3

fn widen(x: int) -> int:
    return [x, x].len()

@noalloc
fn fire(mut b: Bus) -> int:
    return (b.handlers[0])(1)

fn main():
    bus = Bus(handlers = [double, triple])
    log = Log(lines = [widen])
    println(fire(bus))
    println((log.lines[0])(1))
