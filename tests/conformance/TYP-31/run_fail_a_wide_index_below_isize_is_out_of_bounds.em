#$ test: run-fail
#$ rules: TYP-31, TYP-1
#$ profiles: debug, release, shipping
#$ panics: index -18446744073709551617 is out of bounds
# `[TYP-31]` (D-527) — a negative index wider than `isize` that it does not
# hold is past every length too, and is printed whole.

fn main():
    xs = [10, 20, 30]
    at = -(u64.MAX as i128) - 2
    println(xs[at])
