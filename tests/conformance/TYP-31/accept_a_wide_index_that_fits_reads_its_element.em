#$ test: run-pass
#$ rules: TYP-31, TYP-1, STD-19
#$ profiles: debug, release
#$ stdout:
#$ 30 20 30 10
#$ [10, 9, 20, 30]
# `[TYP-31]` (D-527) — an index or a size wider than `usize` (`i128`, `u128`,
# the 256-bit counts) that `usize` holds works as any other.

fn main():
    xs = [10, 20, 30]
    println(xs[2 as u128], xs[1 as i256], xs[2 as u256], xs[0 as i128])
    xs.insert(1 as u128, 9)
    println(xs)
