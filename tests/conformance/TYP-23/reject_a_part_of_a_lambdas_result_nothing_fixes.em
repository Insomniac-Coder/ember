#$ test: compile-fail
#$ rules: TYP-23
# `[TYP-23]` — a part of a lambda's result that neither the expected result nor the body fixes stays open, and one
# still open at the end is `E2060` (it is not `void`): `Err(…)` says the error type and nothing says `U`.

fn main():
    other: Result[int, String] = Ok(2)
    println(other.and_then(owns fn(owns v) => Err(String("no"))))   #$ error[E2060]: cannot tell what `U` is here
