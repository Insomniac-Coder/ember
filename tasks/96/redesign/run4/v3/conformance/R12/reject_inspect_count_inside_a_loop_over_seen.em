#$ test: compile-fail
#$ rules: R12, R8
#$ help: _n = xs.iter().inspect(fn(v) changes seen: seen.push(v)).count()
## R8 (c), R12 (rules.md lines 612-615): driving an adapter that keeps a function which changes `seen`
## is a change of `seen`, refused inside a loop over `seen` (E3020). AN-55 part 6's message, in this
## program's lines.

fn main():
    xs = [1, 2, 3]
    seen: Array[int] = [0]
    for s in seen:
        println(s)
        _n = xs.iter().inspect(fn(v) changes seen: seen.push(v)).count()    #$ error[E3020]: this loop walks `seen` (line 11), and `xs.iter().inspect(...).count()` (line 13) changes `seen` (`count` runs the function `inspect` keeps, which says `changes seen`); count after the loop: write `_n = xs.iter().inspect(fn(v) changes seen: seen.push(v)).count()` after it
