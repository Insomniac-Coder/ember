#$ test: run-pass
#$ rules: CLO-3, FN-6
#$ profiles: debug, release, shipping
#$ stdout: 31 8
#$ stdout: 34 9
#$ assert-c-count: contains("__via_0_em_triple(") == 4
#$ assert-c-count: contains("__via_0_em_closure") == 0
# `[CLO-3]` — a callable parameter is an implicit generic: each function
# passed to it gets `apply`'s own copy, in which the call through the
# parameter is a direct call to that function (one the C compiler can
# inline). `triple` is passed twice, so its copy stays a function and shows
# in the C: declared, defined, and called twice. Each lambda is a function of
# its own, passed once: its copy is inlined where it is called and not emitted.

fn apply(f: fn(int) -> int, v: int) -> int:
    return f(v)

fn triple(x: int) -> int:
    return x * 3 + 1

fn main():
    println(apply(triple, 10), apply(fn(x) => x - 2, 10))
    println(apply(triple, 11), apply(fn(x) => x - 2, 11))
