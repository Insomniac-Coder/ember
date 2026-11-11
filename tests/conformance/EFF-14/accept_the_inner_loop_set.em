#$ test: run-pass
#$ rules: EFF-14, EFF-12, EFF-16
#$ stdout: 10
# `[EFF-14]` — contracts compose: the usual inner-loop set is `@no_alias_checks @noalloc @nosync
# @nopanic(explicit)`, one per line.

@no_alias_checks
@noalloc
@nosync
@nopanic(explicit)
fn sum(xs: Span[int]) -> int:
    total = 0
    for x in xs:
        total += x
    return total

fn main():
    xs: Array[int] = [1, 2, 3, 4]
    println(sum(xs.as_span()))
