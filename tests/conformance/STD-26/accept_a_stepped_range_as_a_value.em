#$ test: run-pass
#$ rules: STD-26
#$ profiles: debug, development
#$ stdout: [10, 7, 4, 1] [0, 4, 8] [] [-3, -1, 1]
#$ [9223372036854775800, 9223372036854775803, 9223372036854775806] [-9223372036854775800, -9223372036854775805]
# `[STD-26]` — `range(start, stop, step)` counts as Python's does, including a negative step, as a
# value too: no step goes past the type's ends.

fn main():
    r = range(10, 0, -3)
    a = [v for v in r]
    b = [v for v in range(0, 10, 4)]
    c = [v for v in range(5, 5, 1)]
    d = [v for v in range(-3, 3, 2)]
    e = [v for v in range(9223372036854775800, 9223372036854775807, 3)]
    f = [v for v in range(-9223372036854775800, -9223372036854775808, -5)]
    println(a, b, c, d)
    println(e, f)
