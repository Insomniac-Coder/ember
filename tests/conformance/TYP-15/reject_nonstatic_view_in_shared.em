#$ test: compile-fail
#$ rules: TYP-15, HEAP-3, RC-1
# A `shares` owner likewise has no bounding region. It must not erase a
# non-static view's provenance merely because the payload is reference counted.
# `[RC-1]` — the local a line shares is made in its block on the line that declares it.

fn main():
    data: [i32; 3] = [1, 2, 3]
    view: Span[i32] = data #$ error[E3063]: `Span[i32]` is a view, so it may not be stored in a block of `shares`
    _shared: shares mut Span[i32] = shares mut view
