#$ test: compile-fail
#$ rules: BCK-4
#$ error[E3060]: `x` does not live long enough
# `[BCK-4]` — `x`'s storage ends with the `if` block while `r` still carries a loan of it that is
# used afterwards: `E3060`.

fn main():
    r: Option[ref int] = None
    if true:
        x: int = 5
        r = Some(ref x)
    println(r.unwrap())
