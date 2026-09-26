#$ test: compile-fail
#$ rules: TYP-15, LT-21, FN-1
# `[TYP-15]` (D-352) — a callee that moves a view from one field of a `mut`
# argument to another moves its region with it: after `swap_fields(p)`,
# `p.a` holds what `p.b` held, so `p.b`'s source is still borrowed while
# `p.a` is used.

struct P:
    a: str
    b: str

fn swap_fields(mut p: P):
    t = p.a
    p.a = p.b
    p.b = t

fn main():
    s = String.from("first")
    t = String.from("second")
    p = P(s.as_str(), t.as_str())
    swap_fields(p)
    mem.drop(t) #$ error[E3021]: `t` cannot be moved while it is borrowed
    println(p.a)
