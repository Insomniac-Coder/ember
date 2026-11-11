#$ test: compile-pass
#$ rules: LNT-2, LNT-3, GRM-4
#$ warning[L1002]: `veloctiy` declares a new binding; `velocity` is already in scope
# `[LNT-2]` — `L1002 assignment declares a new binding`: an unread new name within distance 2 of a
# binding in scope, which is usually a typo. `ember build` and `ember check` report it (`[LNT-3]`).

fn main():
    velocity: i32 = 10
    veloctiy: i32 = 20
    println(velocity)
