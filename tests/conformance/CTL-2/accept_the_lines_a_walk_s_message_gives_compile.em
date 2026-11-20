#$ test: run-pass
#$ rules: CTL-2, BRW-18, DIA-25
#$ stdout: [5, 5]
#$ stdout: [1, 2, 1, 2]
#$ stdout: a 2
# `[CTL-2]`, `[DIA-25]` — the lines `E3020`'s message gives compile: the loop's own handle for an
# element (`for ref mut x in xs:`, `x = 5`) and a map's value (`for k, ref mut v in counts:`, `v *= 2`),
# and a list of your own for a push (`added: Array[int] = []`, `added.push(owns x)`, `xs.extend(owns
# added)`).

fn main():
    xs = [1, 2]
    for ref mut x in xs:
        x = 5
    println(xs)
    ys = [1, 2]
    added: Array[int] = []
    for y in ys:
        added.push(owns y)
    ys.extend(owns added)
    println(ys)
    counts = Map[String, int]()
    counts.insert(owns String("a"), owns 1)
    for _k, ref mut v in counts:
        v *= 2
    for k, v in counts:
        println(k, v)
