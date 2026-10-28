#$ test: compile-fail
#$ rules: R12
#$ help: xs.iter().inspect(fn(v) changes seen: seen.push(v)).count()
## Verdicts 21 and 22, R12 (rules.md lines 729-732): `inspect` keeps the function and `count()` runs it;
## a lambda that pushes to the real `seen` says so. AN-55 part 6's message, word for word.

fn main():
    xs = [1, 2, 3]
    seen: Array[int] = []
    _n = xs.iter().inspect(fn(v) => seen.push(v)).count()    #$ error[E2105]: `inspect` keeps this function and `count()` runs it, and it pushes to `seen`, a variable it uses from `main`; say so on its first line: `xs.iter().inspect(fn(v) changes seen: seen.push(v)).count()`
    println(seen.len())
