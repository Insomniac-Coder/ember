#$ test: compile-fail
#$ rules: FN-2a
#$ help: d = ref mut xs.as_mut_span()
## design rules: R4, R2
## R4, verdict 34: `ref mut` before a plain local that holds a `MutSpan` is refused: the local only
## reads the elements, so this function may not hand them on to be written (AN-14, E3027).

fn zero(ref mut dst: MutSpan[float]):
    for i in 0..dst.len():
        dst[i] = 0.0

fn main():
    xs = [1.0, 2.0]
    d = xs.as_mut_span()
    zero(ref mut d)    #$ error[E3027]: `d` only reads the elements of `xs` (line 13 gives it no `ref mut`), so it cannot be given to `zero` as `ref mut`; write line 13 as `d = ref mut xs.as_mut_span()`
    println(xs[0])
