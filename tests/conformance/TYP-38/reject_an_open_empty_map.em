#$ test: compile-fail
#$ rules: TYP-38, TYP-23
# `[TYP-38]`, `[TYP-23]` — `{}` takes its types from the context or from later
# uses in the function; still open at the end it is `E2060`.

fn main():
    m = {}  #$ error[E2060]: cannot tell what `m` holds
