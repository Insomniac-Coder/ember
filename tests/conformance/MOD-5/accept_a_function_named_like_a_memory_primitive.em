#$ test: run-pass
#$ rules: MOD-5
#$ profiles: debug, development, release
#$ stdout: 2 4 9 0

# `[MOD-5]` (D-560) — a program's own functions named `read`, `write`, `alloc`
# and `free` are its own: the compiler's raw-memory primitives of those names
# took the calls (`read` needed `unsafe` and two arguments).

fn read(x: int) -> int:
    return x + 1

fn write(x: int) -> int:
    return x * 2

fn alloc(x: int) -> int:
    return x * x

fn free(x: int) -> int:
    return x - x

fn main():
    println(read(1), write(2), alloc(3), free(4))
