#$ test: compile-fail
#$ rules: LEX-19, TYP-17, DIA-14
#$ error[E0100]: `:>>q` is not a format spec
# A malformed specification is invalid without knowing the value's type.
# It remains an independent error alongside the missing formatting bound.

fn format_unknown[T](x: T) -> String:
    return f"{x:>>q}"    #$ error[E2040]

fn main():
    pass
