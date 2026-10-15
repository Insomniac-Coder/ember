#$ test: run-pass
#$ rules: BCK-4
#$ stdout: 5
# `[BCK-4]` — a loan whose last use comes before its place's storage ends is no error.

fn main():
    if true:
        x: int = 5
        r: Option[ref int] = Some(ref x)
        println(r.unwrap())
