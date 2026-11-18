#$ test: compile-fail
#$ rules: ATT-2, ATT-6
# `[ATT-2]` — `@allow` is permitted on a statement (and `[ATT-6]`: while its effect is not built it
# is `E0900`, never accepted and ignored).

fn main():
    c = 3
    @allow(L001)                    #$ error[E0900]
    if c == 3:
        println(c)
