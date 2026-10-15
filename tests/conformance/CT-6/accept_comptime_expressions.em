#$ test: run-pass
#$ rules: CT-6, CT-1, TYP-41
#$ stdout: 8 54 80 6
# `[CT-6]` — `comptime(e)` is `e` evaluated while compiling; `e` may name constants, generic
# parameters, and statics initialised at compile time.

fn cube(x: int) -> int:
    return x * x * x

fn scaled[const N: int](x: int) -> int:
    return x * comptime(cube(N))

const TWO: int = 2
static BASE: int = cube(TWO)

fn main():
    println(scaled[2](1), scaled[3](2), comptime(BASE * 10), comptime(TWO + 4))
