#$ test: run-pass
#$ rules: TYP-36, RNG-3
#$ profiles: debug, release, shipping
#$ stdout: true true
# D-318 — a range type hashes as its representation does, so a float range
# does not (`[TYP-36]`). It counted as `Hash`, so `Result[R, RangeError]`
# took `Result`'s `Hash` implementation, whose body cannot hash an `f32`,
# and no method of `R.checked(x)` compiled.

type R = f32 in 0.25 ..= 0.75

fn main():
    println(R.checked(0.6).is_ok(), R.checked(0.9).is_err())
