#$ test: compile-fail
#$ rules: LEX-19, STD-9, STR-5, DIA-14, TYP-39
# Formatting follows printing's fallback and honors an explicit Debug
# opt-out, directly and within an aggregate. A conversion or width spec
# does not add an unrelated invalid-spec diagnostic afterward.

@no_derive(Debug)
struct Secret:
    token: int

fn main():
    secret = Secret(1)
    plain = f"{secret}"    #$ error[E2040]: Debug
    debug = f"{secret!r}"    #$ error[E2040]: Debug
    padded = f"{secret:>8}"    #$ error[E2040]: Debug
    secrets = [Secret(2)]
    nested = f"{secrets}"    #$ error[E2040]: Debug
