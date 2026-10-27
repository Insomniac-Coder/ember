#$ test: compile-fail
#$ rules: R15
## R15: "`static mut NAME: T = e` is a module variable that may change"; Python's module-level
## assignment is refused with that line (AN-76).

FRAME = 0    #$ error[E1030]: `FRAME = 0` at module level is a statement, and this file has `fn main`; declare a module variable: `static mut FRAME: int = 0`

fn main():
    println(1)
