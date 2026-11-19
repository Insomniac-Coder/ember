#$ test: compile-fail
#$ rules: TYP-15, OWN-6, LT-21
# `[TYP-15]` (D-352) — `mem.replace(place, value)` and `mem.swap(a, b)` store
# through their `mut` arguments, and so does a `ref mut`: what they store is
# in the variable afterwards, with the regions it carries.

fn main():
    a = "ann"
    s = String.from("zed")
    old = mem.replace(ref mut a, s.as_str())
    mem.drop(s)
    println(a, old)  #$ error[E3060]: `s` was handed over, and a view of it is still used here

    b = "bob"
    t = String.from("yan")
    v = t.as_str()
    mem.swap(ref mut b, ref mut v)
    mem.drop(t)
    println(b)  #$ error[E3060]: `t` was handed over, and a view of it is still used here

    c = "cy"
    u = String.from("xu")
    r = ref mut c
    r = u.as_str()
    mem.drop(u)
    println(c)  #$ error[E3060]: `u` was handed over, and a view of it is still used here
