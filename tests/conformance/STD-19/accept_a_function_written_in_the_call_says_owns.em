#$ test: run-pass
#$ rules: STD-19, FN-2a
#$ stdout: 1
#$ stdout: 2
#$ stdout: [2, 4] 5
# `[STD-19]` (verdicts 150, 158) — a function written in a call to a standard-library function is
# given to it with `owns`; one from a variable is lent with no word and stays the caller's.

fn main():
    xs = [1, 2]
    xs.iter().for_each(owns fn(x: int): println(x))
    doubled: Array[int] = xs.iter().map(owns fn(x: int) => x * 2).collect()
    h = fn(x: int): pass
    xs.iter().for_each(owns h)
    h(5)
    println(doubled, 5)
