#$ test: run-pass
#$ rules: BRW-12
#$ stdout: 0
## design rules: R2
## R2: before a `MutSpan` a call gives, `ref mut` says the name may write the span's elements
## (verdict 34).

fn main():
    xs = [1, 2]
    d = ref mut xs.as_mut_span()
    d[0] = 0
    println(xs[0])
