#$ test: compile-fail
#$ rules: DIA-15, DIA-24
#$ error[E1010]: cannot find type `sqrt` in this scope
#$ not-help: from std.math import sqrt
#$ error[E1010]: cannot find `Duration` in this scope
#$ help: `Duration` is exported by `std.time`: import it, `from std.time import Duration`
# `[DIA-15]` with `[DIA-24]`: the index of exported names records what each name is, so where a
# type is expected a function another module exports (`std.math.sqrt`) is not offered; where a value
# is expected a type is (`Duration`, whose name constructs and reaches its associated functions).

fn main():
    root: sqrt = 1
    later = Duration.from_millis(5)
