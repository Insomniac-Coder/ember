#$ test: compile-fail
#$ rules: FN-2a
#$ help: blend(const ref mut d, b.as_span())
## design rules: R4
## R4, verdict 70: a call writes the parameter's words, `const` included: for `const ref mut dst`, the
## call says `const ref mut d`, so the line shows that `d` is written and never pointed elsewhere
## (E3028).

fn blend(const ref mut dst: MutSpan[float], src: Span[float]):
    for i in 0..dst.len():
        dst[i] = dst[i] * 0.5 + src[i] * 0.5

fn main():
    a = [1.0, 2.0]
    b = [3.0, 6.0]
    d = ref mut a.as_mut_span()
    blend(ref mut d, b.as_span())    #$ error[E3028]: `blend` takes `dst` as `const ref mut`, which writes the elements of `d` and never points it elsewhere; say so at the call: `blend(const ref mut d, b.as_span())`
    println(a[0], a[1])
