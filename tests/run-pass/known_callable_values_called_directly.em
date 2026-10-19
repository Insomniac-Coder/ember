#$ test: run-pass
#$ profiles: debug, development, release
#$ stdout: 2998 102 6 13 1001
# D-571: a call through a callable value that is a known function, set once
# and never changed, calls that function directly (and it may be inlined and
# its checks proved away). A value set twice, or one holding state, is
# called through as before, as is one swapped through a reference; each
# gives the same answers either way.

from std.mem import swap

fn apply(f: fn(int) -> int, v: int) -> int:
    return f(v)

fn apply_other(f: fn(int) -> int, v: int) -> int:
    return f(v)

fn apply_state(f: fn(int) -> int, v: int) -> int:
    return f(v)

fn apply_swapped(f: fn(int) -> int, v: int) -> int:
    return f(v)

fn twice(x: int) -> int:
    return x * 2

fn main():
    step: fn(int) -> int = fn(x) => x * 3 + 1
    total = 0
    for i in 0..1000:
        total = total + apply(step, i) % 7
    g: fn(int) -> int = twice
    if total > 0:
        g = fn(x) => x + 100
    k = 5
    adder: fn(int) -> int = owned fn(x) => x + k
    calls = 0
    counter: fn(int) -> int = owned fn(x: int) -> int:
        calls = calls + x
        return calls
    counter(6)
    doubled: fn(int) -> int = fn(x) => x * 2
    other: fn(int) -> int = fn(x) => x + 1000
    swap(doubled, other)
    println(total, apply_other(g, 2), apply_state(adder, 1), counter(7), apply_swapped(doubled, 1))
