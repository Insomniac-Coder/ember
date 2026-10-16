#$ test: run-pass
#$ rules: CORO-1, CORO-3, MOD-2
#$ profiles: debug, release
#$ stdout: 1 2
#$ counted 2
#$ 3
# `[CORO-1]` — a public generator, generic, and a public generator method of a generic type,
# called from another module: the frames are the declaring module's types, and `result` is as
# public as the generator.

from generators_support.sequences import Bag, items

fn main():
    data = [1, 2]
    g = items(data)
    a = g.next().unwrap()
    b = g.next().unwrap()
    println(a, b)
    if g.next().is_none():
        println("counted", g.result().unwrap())
    bag = Bag([3])
    for x in bag.each():
        println(x)
