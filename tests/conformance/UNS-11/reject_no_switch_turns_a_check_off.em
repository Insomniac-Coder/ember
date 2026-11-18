#$ test: compile-fail
#$ rules: UNS-11, ATT-1
# `[UNS-11]` — no switch turns a safety check off, in any build or profile: `@no_runtime_checks` is not
# Ember (`E0104`), and the message gives the two ways to write what it was wanted for.

@no_runtime_checks    #$ error[E0104]: there is no switch that turns safety checks off, in any build; to forbid run-time alias checks here write `@no_alias_checks`; to skip the bounds check of an index known to be in range, write `xs.get_unchecked(i)` inside `unsafe:` with a `# SAFETY:` note
fn first(xs: Span[int]) -> int:
    return xs[0]

fn main():
    xs = [1, 2]
    println(first(xs[..]))
