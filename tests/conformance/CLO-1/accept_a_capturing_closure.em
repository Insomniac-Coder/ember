#$ test: run-pass
#$ rules: CLO-1, CLO-2, CLO-3
# "A capturing closure is a unique anonymous struct implementing `Callable`."
# This is that struct: one field per captured name, `[CLO-2]`'s shared borrow
# for a read-only use, and a body that reads through it.
#
# It compiled to a C function pointer until now, which has nowhere to put the
# captures, so every one of these was rejected (ADR-018). `[CLO-3]` says a
# `fn(A) -> R` parameter is "a generic over `Callable`, monomorphised", and
# once it is, the closure's own type is what instantiates the callee.

fn apply(f: fn(i32) -> i32, v: i32) -> i32:
    return f(v)

fn main():
    scale = 3
    println(apply(fn(x) => x * scale, 2))
    offset = 10
    println(apply(fn(x) => x + offset, 5))
#$ stdout: 6
#$ 15
