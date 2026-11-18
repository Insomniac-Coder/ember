#$ test: compile-fail
#$ rules: FN-1a
#$ help: write `ref mut dst: MutSpan[f64]` for the parameter
# `[FN-1a]` — a plain `dst: MutSpan[float]` only reads the elements: a write through it is `E3023`,
# whose message says to write `ref mut` on the parameter.

fn blend(dst: MutSpan[float], src: Span[float]):
    dst[0] += src[0]    #$ error[E3023]: `dst[0] += src[0]` writes an element of the caller's span, but `dst` only reads it; write `fn blend(ref mut dst: MutSpan[float], src: Span[float]):` and call it `blend(ref mut a.as_mut_span(), b.as_span())`

fn main():
    a = [1.0]
    b = [1.0]
    blend(a.as_mut_span(), b.as_span())
