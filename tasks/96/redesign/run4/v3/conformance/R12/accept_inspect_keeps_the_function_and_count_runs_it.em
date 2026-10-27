#$ test: run-pass
#$ rules: R12
#$ stdout: 0
#$ stdout: 3
## R12 (rules.md lines 583-588, 612-615): storing a function in an adapter changes nothing (`seen` is
## still empty after `inspect`); driving the adapter runs it, so `count()` is the call that pushes to
## `seen`. AN-55's iterator line, split in two.

fn main():
    xs = [1, 2, 3]
    seen: Array[int] = []
    it = xs.iter().inspect(fn(v) changes seen: seen.push(v))
    println(seen.len())
    _n = it.count()
    println(seen.len())
