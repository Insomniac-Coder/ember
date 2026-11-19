#$ test: run-pass
#$ rules: RC-1, OWN-13
#$ profiles: debug, development, release
#$ stdout: 3
#$ 4
# `[RC-1]` — an `owns` parameter, which the function owns from its first line, shared by a line of
# it, gets its block at the function's entry, before any view of it is made: its type is the shared
# type throughout the function, and the caller's line is unchanged.

fn count_both(owns xs: Array[int]) -> int:
    n = xs.len()
    other = shares mut xs
    other.push(owns 9)
    println(n)
    return xs.len()

fn main():
    println(count_both(owns [1, 2, 3]))
