#$ test: compile-fail
#$ rules: LT-1, LT-3
# A local view of this frame's data carries that data's loan: returning an
# element through it, or a `ref` of a local, is `E3060` naming the local.

fn bad_span() -> ref int:
    xs = [1, 2]
    s = xs.as_span()
    return ref s[0]    #$ error[E3063]: this points into `xs`

fn bad_ref() -> ref int:
    x = 5
    q: ref int = x
    return q    #$ error[E3063]: this points into `x`

fn main():
    println(bad_span(), bad_ref())
