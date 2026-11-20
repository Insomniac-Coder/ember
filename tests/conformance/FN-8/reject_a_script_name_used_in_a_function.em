#$ test: compile-fail
#$ rules: FN-8
#$ help: a local of the script's implicit `main`, which no function reads
#$ help: a `static mut` for a module variable that functions change
# `[FN-8]` — the names a script's statements bind are locals of the implicit `main`: a function of
# the file that uses one is `E1010` (unlike Python, where it would read a global), whose help names
# a parameter, a `const`, a `static`, or a `static mut`.

limit = 3

fn show():
    println(limit)    #$ error[E1010]: cannot find `limit` in this scope

show()
