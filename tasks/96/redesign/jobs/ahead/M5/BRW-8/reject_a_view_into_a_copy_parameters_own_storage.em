# Clause BRW-8.5: "A view into the own storage of a parameter passed as a copy ends with the call:
# returning it is `E3063`."
#$ test: compile-fail
#$ rules: BRW-8, LT-1
# The spec gives no message text for this case: the expectations name the code only. `w` (a view)
# and `n` (a small `Copy` value) are both passed as a copy.

fn keep(w: StrView) -> ref StrView:
    return ref w    #$ error[E3063]

fn keep_n(n: int) -> ref int:
    return ref n    #$ error[E3063]

fn main():
    println(keep("a"), keep_n(1))
