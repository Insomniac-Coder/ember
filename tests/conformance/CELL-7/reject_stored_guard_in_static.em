#$ test: compile-fail
#$ rules: CELL-7
# `[CELL-7]` + `[TYP-15]` — a guard is a view and may not be stored where it
# outlives its source. A `static` has no bounding region, and a guard's region
# borrows a local cell, so storing one there is `E3063`.

static G: Ref[i32] = 1   #$ error[E3063]: may not be stored
