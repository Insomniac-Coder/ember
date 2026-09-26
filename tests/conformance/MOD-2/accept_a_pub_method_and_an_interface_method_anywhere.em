#$ test: run-pass
#$ rules: MOD-2
#$ stdout: 5 5 6 7
# `[MOD-2]` — a `pub` method or associated function is callable from any
# module; a method implementing a public interface is what the interface
# offers, and is callable wherever the interface is visible, `pub` or not.

from support.methods import Meter, Box2

fn main():
    m = Meter.make(5)
    b = Box2(7)
    println(m.read(), m.value, m.shown(), b.get())
