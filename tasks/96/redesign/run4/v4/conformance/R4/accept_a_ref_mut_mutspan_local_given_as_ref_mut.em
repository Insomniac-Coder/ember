#$ test: run-pass
#$ rules: R4, R2
#$ stdout: 0.0
## R4, verdict 34: a local made with `ref mut` from `as_mut_span()` may let its elements be written,
## and is given to the call as `ref mut d` (AN-14).

fn zero(ref mut dst: MutSpan[float]):
    for i in 0..dst.len():
        dst[i] = 0.0

fn main():
    xs = [1.0, 2.0]
    d = ref mut xs.as_mut_span()
    zero(ref mut d)
    println(xs[0])
