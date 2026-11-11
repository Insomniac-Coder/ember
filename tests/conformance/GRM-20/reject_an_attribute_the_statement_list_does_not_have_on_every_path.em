#$ test: compile-fail
#$ rules: GRM-20, ATT-3
# `[GRM-20]` — the attributes before a statement are limited to `@parallel`, `@unroll`, `@simd` and
# `@allow`, wherever the statement stands: as Python's own statements, and with no statement after
# the attribute at all.

fn main():
    @inline                         #$ error[E0108]: `@inline` is not permitted on a statement
    raise ValueError("no")          #$ error[E0100]: `raise` is Python's
    @inline                         #$ error[E0108]: `@inline` is not permitted on a statement
    global count                    #$ error[E0100]: `global` is Python's
    if true:
        @simd                       #$ error[E0108]: `@simd` must precede a compound statement
