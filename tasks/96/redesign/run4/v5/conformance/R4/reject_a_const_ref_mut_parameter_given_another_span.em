#$ test: compile-fail
#$ rules: BRW-14, BRW-15
#$ help: fn fill(ref mut dst: MutSpan[float], ref mut spare: MutSpan[float]):
## design rules: R4, R2
## R4, verdict 42: a `const ref mut` parameter writes the caller's elements and never points the
## caller's span elsewhere; giving it another ref is refused (E3023).

fn fill(const ref mut dst: MutSpan[float], ref mut spare: MutSpan[float]):
    dst = ref mut spare    #$ error[E3023]: `dst` is `const ref mut`, so it always refers to the span `main` gave; write `spare`'s elements through `spare`: `spare[0] = 1.0` in place of this line; or make the parameter plain `ref mut`: `fn fill(ref mut dst: MutSpan[float], ref mut spare: MutSpan[float]):`, called `fill(ref mut a.as_mut_span(), ref mut b.as_mut_span())`
    dst[0] = 1.0

fn main():
    a = [0.0]
    b = [0.0]
    fill(const ref mut a.as_mut_span(), ref mut b.as_mut_span())
    println(a[0], b[0])
