# Clause BRW-8.2: "Two kinds are passed as a copy instead: a view (a `ref`, `Span`, `MutSpan`,
# `StrView` or `@view` struct), which is passed as itself and keeps the regions it carries"
#$ test: run-pass
#$ rules: BRW-8, LT-1, BRW-13
#$ stdout: hel
# `head`'s result points into what `w` views (`text`), not into `w`: it lives on after `w`'s block.

fn head(w: StrView) -> StrView:
    return w[0..3]

fn main():
    text = String("hello world")
    first: StrView = "none"
    if text.len() > 3:
        w: StrView = text
        first = head(w)
    println(first)
