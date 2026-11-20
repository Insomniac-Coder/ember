#$ test: compile-fail
#$ rules: CT-1a, STA-3, STA-1
# `[CT-1a]` — what compile time evaluates reaches no module variable that lives at run time: a
# `static mut` named in `comptime(e)` is `E6010` (a static's own initialiser that names one runs on
# first access instead, `[STA-3]`).

static mut FRAME: int = 3

fn main():
    x = comptime(FRAME + 1)    #$ error[E6010]: `FRAME` is a `static mut`, and a value made when compiling cannot see what the program will hold while running
    println(x)
