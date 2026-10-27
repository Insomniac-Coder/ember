#$ test: run-pass
#$ rules: R12
#$ stdout: 0
#$ stdout: 3
## R12 (rules.md lines 711-714, 735-739): "An iterator adapter keeps the function it is given and runs it
## at each step"; storing it in the adapter changes nothing (`seen` is still empty after `inspect`), and
## driving the adapter runs it, so `count()` is the call that pushes to `seen`. AN-55's iterator line,
## split in two; nothing on the lambda says what it changes (verdict 86).

fn main():
    xs = [1, 2, 3]
    seen: Array[int] = []
    it = xs.iter().inspect(fn(v): seen.push(v))
    println(seen.len())
    _n = it.count()
    println(seen.len())
