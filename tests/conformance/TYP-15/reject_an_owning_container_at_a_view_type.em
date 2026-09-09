#$ test: compile-fail
#$ rules: TYP-15, TYP-15a
# The scope limit the owner set: ERR-044's decision is about a view's *region*,
# and it does not touch `[TYP-15a]`. An ordinary owning container instantiated
# at a view type stays rejected whatever the region — that rejection is at the
# type, not at the region, and `BorrowList[T]`/`ViewList[T]` remain the
# specialised model for holding views.

fn main():
    xs: Array[str] = Array[str]()    #$ error[E3063]: `str` is a view, so it may not be stored in a container element
    println(1)
