#$ test: compile-fail
#$ rules: EFF-2, EFF-5
# `[EFF-2]` (verdict 129, D-930) — a function put into a list (a list written out, `push`) is followed
# into the list's place: `fire` calls what `bus.handlers` holds, and `push` put `widen`, which
# allocates, there. (Before, the list's own code put it there unseen, and the call ran nothing.)

struct Bus:
    handlers: Array[fn(int) -> int]

fn double(x: int) -> int:
    return x * 2

fn widen(x: int) -> int:
    return [x, x].len()

@noalloc
fn fire(mut b: Bus) -> int:
    return (b.handlers[0])(1)    #$ error[E4001]: `@noalloc` function `fire` reaches an allocation

fn main():
    bus = Bus(handlers = [double])
    bus.handlers.push(widen)
    println(fire(bus))
