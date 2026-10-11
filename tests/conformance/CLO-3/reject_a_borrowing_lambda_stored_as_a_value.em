#$ test: compile-fail
#$ rules: CLO-3, CLO-4, TYP-15

# `[CLO-3]` (D-554) — a lambda that borrows is a view, and a `fn` type outside
# a parameter is an owned callable value: storing one there is `E3026` (shape
# B9, ODR-103), whose help is `owned fn`. It used to name the compiler's
# environment type.

struct Holder:
    f: fn() -> int

fn main():
    s = "hi".to_string()
    h = Holder(fn() => s.len())    #$ error[E3026]: this lambda borrows what it captures, so it may not be stored as `fn() -> i64`
    println(h.f())
