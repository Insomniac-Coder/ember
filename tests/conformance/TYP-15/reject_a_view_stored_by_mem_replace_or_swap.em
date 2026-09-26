#$ test: compile-fail
#$ rules: TYP-15, OWN-6, LT-21
# `[TYP-15]` (D-352) — `mem.replace(place, value)` and `mem.swap(a, b)` store
# through their `mut` arguments, and so does a `ref mut`: what they store is
# in the variable afterwards, with the regions it carries.

fn main():
    a = "ann"
    s = String.from("zed")
    old = mem.replace(a, s.as_str())
    mem.drop(s) #$ error[E3021]: `s` cannot be moved while it is borrowed
    println(a, old)

    b = "bob"
    t = String.from("yan")
    v = t.as_str()
    mem.swap(b, v)
    mem.drop(t) #$ error[E3021]: `t` cannot be moved while it is borrowed
    println(b)

    c = "cy"
    u = String.from("xu")
    r = ref mut c
    r = u.as_str()
    mem.drop(u) #$ error[E3021]: `u` cannot be moved while it is borrowed
    println(c)
