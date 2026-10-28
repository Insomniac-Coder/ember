#$ test: compile-fail
#$ rules: ATT-1, PHIL-12
## design rules: R16
## R16: the contract is spelled by what it forbids, `@no_alias_checks` (AN-81's message; section 3's
## removed words, E0104).

@static_safe    #$ error[E0104]: `@static_safe` is now `@no_alias_checks`: it forbids run-time alias checks only; bounds and overflow checks still run; write `@no_alias_checks`
fn total(xs: Span[int], n: int) -> int:
    s = 0
    for i in 0..n:
        s += xs[i]
    return s

fn main():
    xs = [1, 2, 3]
    println(total(xs.as_span(), 3))
