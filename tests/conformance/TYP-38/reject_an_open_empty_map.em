#$ test: compile-fail
#$ rules: TYP-38
# `[TYP-38]` — `{}` needs its types from the context; left open it is `E2060`.

fn main():
    m = {}  #$ error[E2060]: cannot tell what `{}` holds
