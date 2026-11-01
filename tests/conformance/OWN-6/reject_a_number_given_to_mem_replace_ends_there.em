#$ test: compile-fail
#$ rules: OWN-6, OWN-9
# `[OWN-6]` — `mem.replace(ref mut place: T, owns new: T)`: the number given as `new` is handed over,
# as to every `owns` parameter (`[OWN-9]`).
import std.mem

fn main():
    a = 1
    b = 2
    old = mem.replace(a, b)
    println(old, a)
    println(b)  #$ error[E3040]
