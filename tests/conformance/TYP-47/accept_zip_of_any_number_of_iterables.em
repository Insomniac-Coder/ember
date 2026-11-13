#$ test: run-pass
#$ rules: TYP-47, STD-26, TYP-27
#$ profiles: debug, development, release
#$ stdout: 1 10 a
#$ 2 20 b
#$ 1 10 a true
#$ 2 20 b false
#$ [(1, 10, 'a'), (2, 20, 'b')]
#$ ['ann1a', 'bo2b']
#$ 1
#$ 2
#$ 3
#$ [1, 2, 3]
# `[STD-26]`, `[TYP-47]` — `zip(a, b, …)` of any number of iterables gives one flat tuple of an item
# of each, stopping at the shortest; `zip(xs)` of one gives each item plainly, there being no
# one-element tuple (`[TYP-27]`). In a `for` header over lists it is one counted loop, as of two.

fn main():
    xs = [1, 2, 3]
    ys = [10, 20]
    cs = ['a', 'b', 'c']
    for x, y, c in zip(xs, ys, cs):
        println(x, y, c)
    flags = [true, false, true]
    for x, y, c, f in zip(xs, ys, cs, flags):
        println(x, y, c, f)
    triples = [(x, y, c) for (x, y, c) in zip(xs.iter().copy_each(), ys.iter().copy_each(), cs.iter().copy_each())]
    println(triples)
    names: Array[String] = ["ann", "bo"]
    joined = [f"{n}{x}{c}" for (n, x, c) in zip(names.iter(), xs.iter(), cs.iter())]
    println(joined)
    for x in zip(xs):
        println(x)
    plain = [x for x in zip(xs.iter().copy_each())]
    println(plain)
