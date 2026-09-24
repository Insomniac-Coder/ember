#$ test: compile-fail
#$ rules: STD-26
# `[STD-26]` — only `a..b` and `a..=b` over integers have a length. A stepped
# `range` is built as a `for` loop head, not yet as a value.

fn main():
    a = len(0..)            #$ error[E2040]: `RangeFrom[i64]` has no length
    b = len(0.5..1.5)       #$ error[E2040]: `Range[f64]` has no length
    c = (0..3).len(1)       #$ error[E2020]: `len` takes 0 argument(s), found 1
    d = range(0, 10, 2)     #$ error[E0900]: a `range` with a step as a value is not implemented yet
