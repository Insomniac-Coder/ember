#$ test: compile-fail
#$ rules: STA-1, GRM-2
#$ error[E0100]: a statement at file scope outside the entry file
#$ help: `FRAME = 0` is a statement, and a module holds declarations; for a module variable that may change write `static mut FRAME: int = 0`
# `[STA-1]` — a statement at module level outside the entry file that would give a module variable
# a value is `E0100`, whose help gives the declaration.

from module_support.frames import first

fn main():
    println(first())
