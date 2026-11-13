#$ test: compile-fail
#$ rules: CLO-2, CLO-11

# `[CLO-2]` (ODR-102) — an owned callable value may write the state it holds,
# so it is called from a mutable place; through a borrowed parameter it is
# `E3023`, whose help is to take the parameter `mut`.

struct Holder:
    f: fn() -> int

fn through(h: Holder) -> int:
    return h.f()    #$ error[E3023]: `through` sets `f`, but `h` only reads the caller's holder; write `fn through(ref mut h: Holder) -> int:` and call `through(ref mut h)`

fn main():
    h = Holder(fn() => 1)
    println(through(h))
