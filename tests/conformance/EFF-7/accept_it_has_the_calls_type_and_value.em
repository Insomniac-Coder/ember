#$ test: run-pass
#$ rules: EFF-7
#$ stdout: 3
# `[EFF-7]` — `@assume_noalloc(call)` is an expression with the call's type and value.

fn total(xs: Span[int]) -> int:
    copy: Array[int] = []
    for x in xs:
        copy.push(x)
    return copy.len()

@noalloc
fn count(xs: Span[int]) -> int:
    n: int = 0
    unsafe:  # SAFETY: a test of the expression's type and value; the claim is the test's to keep
        n = @assume_noalloc(total(xs))
    return n

fn main():
    values = [1, -2, 3]
    println(count(values.as_span()))
