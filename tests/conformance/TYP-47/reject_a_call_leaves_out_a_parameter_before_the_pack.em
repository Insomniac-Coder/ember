#$ test: compile-fail
#$ rules: TYP-47, TYP-25
## `[TYP-47]`, `[TYP-25]` (N171) - a function over a pack takes its positional arguments first for the
## parameters before the pack's values: leaving one out is `E2021`, naming that parameter, not the named-only
## parameter after the values.

[Ts...: needs Printable]
fn log(prefix: StrView, values: Ts..., sep: StrView = " "):
    println(prefix, values..., sep=sep)

fn main():
    log()    #$ error[E2021]: `log()` gives nothing for `prefix`, which has no default
