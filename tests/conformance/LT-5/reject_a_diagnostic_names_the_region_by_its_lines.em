# Clause LT-5.2: "Diagnostics describe them in source terms: "the view of `x` on line 12 is still
# used on line 19"."
#$ test: compile-fail
#$ rules: LT-5, BRW-1, DIA-7
# The region is named by its lines, as `[BRW-1]`'s message names them: where the view was made and
# where it is still used. Only those parts of the text are pinned.

fn main():
    x = 1
    r = ref x
    x = 5    #$ error[E3021]: (`r = ref x`, line 10), and `r` is used on line 12
    println(r)
