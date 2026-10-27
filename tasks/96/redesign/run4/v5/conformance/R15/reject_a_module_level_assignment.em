#$ test: compile-fail
#$ rules: GRM-2
## design rules: R15
## R15: "`static mut NAME: T = e` is a module variable that may change"; Python's module-level
## assignment is refused with that line: E0100, "a statement at module level, such as `FRAME = 0`"
## (section 4; AN-76's E1030 for it became E0100). Section 3's message.

FRAME = 0    #$ error[E0100]: `FRAME = 0` is a statement, and a module holds declarations; for a module variable that may change write `static mut FRAME: int = 0`

fn main():
    println(1)
