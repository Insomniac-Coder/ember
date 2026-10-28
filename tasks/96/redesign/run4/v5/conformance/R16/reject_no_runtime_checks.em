#$ test: compile-fail
#$ rules: ATT-1, PHIL-12
## design rules: R16
## R16: "No switch turns safety checks off" (AN-82's message; section 3's removed words, E0104).

@no_runtime_checks    #$ error[E0104]: there is no switch that turns safety checks off, in any build (`[PHIL-13]`); to forbid run-time alias checks here write `@no_alias_checks`; to skip one bounds check you have proved, write `xs.get_unchecked(i)` inside `unsafe:` with a `# SAFETY:` note
fn total(xs: Span[int], n: int) -> int:
    s = 0
    for i in 0..n:
        s += xs[i]
    return s

fn main():
    xs = [1, 2, 3]
    println(total(xs.as_span(), 3))
