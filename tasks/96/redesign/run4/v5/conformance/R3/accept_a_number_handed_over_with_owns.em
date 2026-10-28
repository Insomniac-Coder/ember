#$ test: run-pass
#$ rules: OWN-8
#$ stdout: 5
## design rules: R3
## R3 (verdict 84): `b = owns n` hands the number to `b`; `b` holds it, and `n` is not used again.

fn main():
    n = 5
    b = owns n
    println(b)
