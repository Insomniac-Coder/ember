#$ test: compile-fail
#$ rules: STD-18, LEX-19
#$ error[E2250]: `format`'s spec is not a string literal
#$ error[E2250]: format spec `:d` does not apply to `f64`
# `[STD-18]` — `format`'s spec is checked against the value's type where it is written, as an
# f-string's is (`E2250`): one that is not a literal cannot be, and one that does not apply is
# refused.

fn main():
    spec = ".2f"
    println(format(2.5, spec))
    println(format(2.5, "d"))
