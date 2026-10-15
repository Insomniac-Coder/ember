#$ test: compile-fail
#$ rules: STD-26
#$ error[E2020]: `sorted` has no argument named `cmp`
# `[STD-26]` — `sorted` takes `key` and `reverse`.

fn main():
    println(sorted([2, 1], cmp=fn(a, b) => a < b))
