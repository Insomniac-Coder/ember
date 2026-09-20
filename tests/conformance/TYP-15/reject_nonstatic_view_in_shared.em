#$ test: compile-fail
#$ rules: TYP-15, HEAP-3
# A Shared owner likewise has no bounding region. It must not erase a
# non-static view's provenance merely because the payload is reference counted.

fn main():
    data = [1, 2, 3]
    view: Span[i32] = data
    _shared: Shared[Span[i32]] = Shared(view) #$ error[E3063]: `Span[i32]` is a view, so it may not be stored in a Shared's contents
