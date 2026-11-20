# Clause EFF-12.2: "`@static_safe` is `E0104` ("`@static_safe` is now `@no_alias_checks`: it forbids
# run-time alias checks only; bounds and overflow checks still run; write `@no_alias_checks`")."
#$ test: compile-fail
#$ rules: EFF-12, ATT-1
# The spec's message. `tests/conformance/ATT-1/reject_attributes_gone_from_ember.em` pins another
# text for the same line (§2).

@static_safe    #$ error[E0104]: `@static_safe` is now `@no_alias_checks`: it forbids run-time alias checks only; bounds and overflow checks still run; write `@no_alias_checks`
fn first(xs: Span[int]) -> int:
    return xs[0]

fn main():
    println(first([1]))
