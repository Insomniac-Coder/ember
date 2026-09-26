#$ test: compile-fail
#$ rules: MOD-2
# `[MOD-2]` — a method or associated function without `pub` is private to its
# module, as a field is, and so is a method implementing a private interface.
# D-328: each of these was callable from any module.

from support.methods import Meter, Box2

fn main():
    m = Meter.make(5)
    b = Box2(7)
    println(m.twice()) #$ error[E1052]: `twice` is private to `support.methods`
    println(Meter.raw(1).value) #$ error[E1052]: `raw` is private to `support.methods`
    println(m.hidden()) #$ error[E1052]: `hidden` is private to `support.methods`
    println(b.peek()) #$ error[E1052]: `peek` is private to `support.methods`
