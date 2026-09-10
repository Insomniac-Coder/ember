#$ test: compile-fail
#$ rules: CELL-7
# `[CELL-7]` — escaping vs non-escaping: a guard borrowing a by-value
# parameter's copy cannot be returned. The copy's storage ends with the frame,
# so the borrow would dangle (`E3060`, borrowing `[LT-1]`'s elision which ties
# returns to view-typed parameters only).

fn get(c: RefCell[i32]) -> Ref[i32]:
    return c.borrow()   #$ error[E3060]: does not live long enough
