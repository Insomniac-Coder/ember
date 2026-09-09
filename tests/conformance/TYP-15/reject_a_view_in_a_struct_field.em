#$ test: compile-fail
#$ rules: TYP-15, TYP-14
## A struct carrying a borrow **is** a view type, and the attribute is required
## as documentation.

struct Holder:
    s: Span[i32]                   #$ error[E2030]: `Holder` carries a borrow, so it is a view type
fn main(): pass
