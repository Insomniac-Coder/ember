#$ test: run-pass
#$ rules: FN-6, STA-1
#$ stdout: 14
#$ stdout: 14
#$ stdout: 9
# `[FN-6]` — a function is a value, and a `static` may hold one: calling the static by name calls
# the function it holds, as calling a local of function type does (D-692: `HOOK(7)` was `E1010`,
# though `h = HOOK` then `h(7)` worked). A local of the same name shadows the static.

extern "C" fn double_it(x: i32) -> i32:
    return x * 2

static HOOK: extern "C" fn(i32) -> i32 = double_it

fn shadowed() -> i32:
    HOOK = fn(x: i32) => x + 2      # a local named like the static
    return HOOK(7)

fn main():
    println(HOOK(7))
    h = HOOK
    println(h(7))
    println(shadowed())
