#$ test: run-pass
#$ rules: OWN-9, OWN-8
#$ stdout: 1
#$ 2
#$ 1
#$ 5
#$ [1, 2] 7
# `[OWN-9]` — where a name only views a number, `copy of` gives an `owns` parameter one of its own:
# a loop's name, an element, a plain parameter. `m[k] = n` is an `=` line, which copies the number,
# whatever `index_set` takes.

fn take(owns x: int):
    println(x)

fn pass_on(n: int):
    take(copy of n)

fn record(ref mut m: Map[StrView, int], name: StrView, n: int):
    m[name] = n

fn main():
    xs = [1, 2]
    for x in xs:
        take(copy of x)
    take(copy of xs[0])
    pass_on(5)
    m: Map[StrView, int] = {}
    record(ref mut m, "a", 7)
    println(xs, m["a"])
