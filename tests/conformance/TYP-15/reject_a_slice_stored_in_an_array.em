#$ test: compile-fail
#$ rules: TYP-15
#$ error[E3063]: `Span[i64]` is a view, so it may not be stored in an `Array` unless it is `static`
# D-199 — an `Array`'s elements have no bounding region, so a view stored in
# one must be `static`. The inferred list literal used to skip the check, and
# reassigning `xs` then freed what `vs[0]` pointed into.

fn main():
    xs = [1, 2, 3, 4, 5, 6, 7, 8]
    vs = [xs[2..]]
    xs = [0]
    println(vs[0])
