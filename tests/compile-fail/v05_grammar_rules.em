#$ test: compile-fail
#$ rules: GRM-15, GRM-18, GRM-19, ATT-2, ATT-3
#$ error[E0105]: `;` is not a statement separator
#$ error[E2036]: this pattern always matches
#$ error[E0109]: `owned` is not permitted in expression position
#$ error[E0108]: must precede a compound statement
#$ error[E0104]: is not permitted on a statement

fn main():
    ## `[GRM-18]` (`OQ-25`) — one line carries one statement.
    a: i32 = 1; b: i32 = 2

    ## `[GRM-19]` — a condition's pattern must be refutable.
    if x = a:
        println(x)

    ## `[GRM-15]` — `owned` marks a parameter, a receiver, a closure or a
    ## consumed scrutinee, and nothing else.
    c: i32 = owned b

    ## `[ATT-3]` — a statement attribute attaches to a compound statement.
    @simd
    c = c + 1

    ## `[ATT-2]` — and only four attributes may appear there at all.
    @inline
    while c > 0:
        c = c - 1
