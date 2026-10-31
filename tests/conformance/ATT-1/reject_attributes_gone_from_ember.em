#$ test: compile-fail
#$ rules: ATT-1, LT-1a
## `[ATT-1]` — an attribute Ember no longer has is `E0104`, with the one to write: `@static_safe` is
## `@no_alias_checks`, `@fp(contract)` is `@fp(allow_fma)`, `@borrows(p)` is `from p` after the
## result type (`[LT-1a]`), and no switch turns a safety check off.

@static_safe    #$ error[E0104]: `@static_safe` is gone
fn one() -> int:
    return 1

@fp(contract)    #$ error[E0104]: `@fp(contract)` is gone
fn two(a: f64) -> f64:
    return a * 2.0 + 1.0

@borrows(a)    #$ error[E0104]: `@borrows` is gone
fn pick(a: Span[int], b: Span[int]) -> Span[int]:
    return a

@no_runtime_checks    #$ error[E0104]: `@no_runtime_checks` is gone
fn three() -> int:
    return 3

fn main():
    xs = [1]
    println(one(), two(1.0), pick(xs.as_span(), xs.as_span()).len(), three())
