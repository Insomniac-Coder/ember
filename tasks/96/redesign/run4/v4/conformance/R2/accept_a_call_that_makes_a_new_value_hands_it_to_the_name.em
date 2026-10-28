#$ test: run-pass
#$ rules: R2
#$ stdout: 2
## R2: a call whose result is a new value the callee made hands that value to the name, which owns
## it and may change it.

fn make() -> Array[int]:
    return [1]

fn main():
    xs = make()
    xs.push(2)
    println(xs.len())
