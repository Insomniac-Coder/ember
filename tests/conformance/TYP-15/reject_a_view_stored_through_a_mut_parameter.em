#$ test: compile-fail
#$ rules: TYP-15, FN-1, LT-21
# `[TYP-15]` (D-352) — a `mut` parameter is the caller's place, so a view the
# callee stores in it is the caller's afterwards: `put` stores `v` in `name`,
# and `name` then borrows what `v` borrows. Dropping `s` while `name` is still
# used is refused, as it would be had `main` written `name = s.as_str()`.
# Before D-352 this compiled and printed freed memory.

fn put(mut x: str, v: str):
    x = v

fn main():
    name = "ann"
    s = String.from("zed")
    put(name, s.as_str())
    mem.drop(s) #$ error[E3021]: `s` cannot be moved while it is borrowed
    println(name)
