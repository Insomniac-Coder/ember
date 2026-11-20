#$ test: compile-fail
#$ rules: SPN-3
#$ error[E2072]: `Span[i32]` has no method named `reborrow`
#$ error[E2072]: `MutSpan[i32]` has no method named `reborrow`
#$ help: a span is lent with `ref mut`: write `ref mut span` in place of `span.reborrow()` [SPN-3]

# `[SPN-3]` (0.9.15) — a span is lent with `ref mut` before it, and a `Span` is `Copy`: `reborrow()` is no
# method of either.

fn main():
    values: Array[i32] = Array[i32]()
    values.push(owns 1)
    shared = values.as_span()
    _other = shared.reborrow()
    span = ref mut values.as_mut_span()
    _again = span.reborrow()
