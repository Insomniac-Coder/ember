# Clause CLO-10.1: "Its state is what it needs besides its code: what it captured with a word, in one
# block of its own made with the value (it carries `Alloc` (...)), except that a state of one word or
# less that the function's body never writes is kept in the state word itself"; "its `Alloc` is at
# the line that makes the function value, (...) and `E4001` there names it"; "one that uses real
# variables and also captured something with a word keeps the frame's address and its captures
# together in a block of its own"
#$ test: compile-fail
#$ rules: CLO-10, EFF-5, EFF-24
# The spec's two messages, with this file's names and line numbers.

class P:
    vy: float = 1.0

@noalloc
fn counter() -> fn():
    n = 0
    return fn() owns n: n += 1    #$ error[E4001]: `counter` is `@noalloc`, and the function made on line 17 keeps `n`, which it changes (`n += 1`), in a block of its own; keep the count in the caller's variable instead: `fn counter(ref mut n: int)`

@noalloc
fn fall(ref mut ps: Array[P], dt: float, k: float):
    step: fn(ref mut P) = fn(ref mut p) copy of k: p.vy *= k * dt    #$ error[E4001]: `fall` is `@noalloc`, and the function made on line 21 uses the real `dt` and keeps its own copy of `k`, so its state is two words and lives in a block of its own; use the real `k` too: `fn(ref mut p): p.vy *= k * dt`
    for ref mut q in ps:
        step(ref mut q)

fn main():
    c = counter()
    c()
    ps = [P()]
    fall(ref mut ps, 0.5, 2.0)
