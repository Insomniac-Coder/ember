#$ test: compile-fail
#$ rules: TYP-18, DIA-14
# `[DIA-14]` — one mistake, one error: a generic constructor's argument is
# checked once, so an error in it is reported once (D-236).

[F]
struct K:
    f: F

fn main():
    k = K(f=nope + 1)       #$ error[E1010]: cannot find `nope` in this scope
