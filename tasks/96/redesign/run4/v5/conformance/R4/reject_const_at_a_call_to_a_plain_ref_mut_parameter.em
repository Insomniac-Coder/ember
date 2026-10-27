#$ test: compile-fail
#$ rules: FN-2a
#$ help: zero(ref mut d)
## design rules: R4
## R4, verdict 70: `const` at the call is refused where the parameter does not say it: a plain
## `ref mut dst` may point the caller's span elsewhere, which `const` would deny (E3028).

fn zero(ref mut dst: MutSpan[float]):
    for i in 0..dst.len():
        dst[i] = 0.0

fn main():
    xs = [1.0, 2.0]
    d = ref mut xs.as_mut_span()
    zero(const ref mut d)    #$ error[E3028]: `zero` takes `dst` as `ref mut`, which may also point `d` at another span, so the call cannot promise `const`; write the parameter's words: `zero(ref mut d)`
    println(xs[0])
