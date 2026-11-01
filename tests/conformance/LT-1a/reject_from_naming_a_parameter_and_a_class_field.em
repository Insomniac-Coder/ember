#$ test: compile-fail
#$ rules: LT-1a
# `[LT-1a]` — a bare field name names any field of `this`, a class's too: a name that is both a
# parameter and a field of the class is `E2118`, as for a struct.

class Doc:
    log: Array[int]

    fn first(log: Array[int]) -> ref int from log:    #$ error[E2118]: `log` is both a parameter of `first` and a field of `Doc`, so `from log` could mean either
        return ref log[0]

fn main():
    d = Doc(log=[1])
    ys = [3]
    println(d.first(ys))
