#$ test: compile-pass
#$ rules: LNT-1, LNT-2, LNT-3, GRM-4
#$ warning[L1001]: `unused` is never read
#$ warning[L1002]: `veloctiy` declares a new binding; `velocity` is already in scope

## `[GRM-4]` makes `x = e` declare when `x` is not in scope and assign when it
## is, so a typo in the name silently makes a second variable. `L1002` is what
## catches that; `L1001` is what makes it visible at all.

fn main():
    velocity: i32 = 10
    unused: i32 = 3
    _ignored: i32 = 4
    veloctiy: i32 = 20
    println(velocity)
