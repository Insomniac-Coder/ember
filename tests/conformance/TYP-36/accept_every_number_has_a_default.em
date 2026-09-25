#$ test: run-pass
#$ rules: TYP-36
#$ stdout: 0 0 0.0 0 0 0.0
# `[TYP-36]` — every number's `Default` is its zero: `i128`, `u128` and `f16`
# too (D-332: those three had none, so a `T: Default` bound refused them).

fn zero[T: Default]() -> T:
    return T.default()

fn main():
    println(zero[i128](), zero[u128](), zero[f16](), i128.default(), u128.default(), f16.default())
