#$ test: run-fail
#$ rules: TYP-31, TYP-1
#$ profiles: debug, release, shipping
#$ panics: index 18446744073709551617 is out of bounds
# `[TYP-31]` (D-527) — an index of any integer type: one wider than `usize`
# that `usize` does not hold is past every length. `as` kept its low bits, so
# this read `xs[1]`.

fn main():
    xs = [10, 20, 30]
    at = (u64.MAX as u128) + 2
    println(xs[at])
