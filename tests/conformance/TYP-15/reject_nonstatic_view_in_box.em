#$ test: compile-fail
#$ rules: TYP-15, TYP-15a, DRP-6
# A heap owner has no bounding region. Boxing a non-static view must not turn
# that view into owned storage or erase its provenance.

fn main():
    data: [i32; 3] = [1, 2, 3]
    view: Span[i32] = data
    boxed: Box[Span[i32]] = Box(view) #$ error[E3063]: `Span[i32]` is a view, so it may not be stored in a Box's contents
