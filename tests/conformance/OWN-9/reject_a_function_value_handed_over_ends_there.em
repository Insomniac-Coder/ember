#$ test: compile-fail
#$ rules: OWN-9, OWN-6, CLO-3
# `[OWN-9]`, `[OWN-6]` — `owns` hands over a value of every type, a function value too: given to an
# `owns` parameter of the program's own, or to `mem.drop`, the name ends there.
import std.mem

fn run(owns f: fn()):
    f()

fn main():
    g = fn(): println(1)
    run(owns g)
    g()    #$ error[E3040]
    h = fn(): println(2)
    mem.drop(h)
    h()    #$ error[E3040]
