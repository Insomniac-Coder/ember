# Clause BRW-11.1: "unless the compiler proves they differ: different constants, `j` from `i + 1..`,
# inside `if i != j:`, or after `assert_disjoint(a, b)` (with `from std.mem import
# assert_disjoint`, `[DSJ-5]`)"
#$ test: run-pass
#$ rules: BRW-11, BRW-5, DSJ-5
#$ stdout: 6 2 3
#$ stdout: 8 2 3
#$ stdout: 10 2 3
# Three of the proofs before a call with two `ref mut` elements of one list; `assert_disjoint` is
# accept_two_spans_of_one_list_proved_apart_or_copied's.

fn pair(ref mut a: int, ref mut b: int):
    a += b

fn main():
    xs = [1, 2, 3]
    pair(ref mut xs[0], ref mut xs[1])
    pair(ref mut xs[0], ref mut xs[2])
    println(xs[0], xs[1], xs[2])
    for i in 0..1:
        for j in i + 1..2:
            pair(ref mut xs[i], ref mut xs[j])
    println(xs[0], xs[1], xs[2])
    i = 1
    j = 0
    if i != j:
        pair(ref mut xs[j], ref mut xs[i])
    println(xs[0], xs[1], xs[2])
