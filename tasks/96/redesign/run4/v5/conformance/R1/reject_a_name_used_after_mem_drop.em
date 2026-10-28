#$ test: compile-fail
#$ rules: OWN-3
#$ help: use `a` before line 12, or remove the `mem.drop`
## design rules: R1
## R1: `mem.drop(a)` ends the name `a` now, for every type, a number included (AN-43; the `mem.drop`
## row of `V4/vocabulary.md`); a later use is E3040 (`V4/rules.md` section 4: "a name used after it was
## handed over or ended").

import std.mem

fn main():
    a = 5
    mem.drop(a)
    println(a)                 #$ error[E3040]: `a` ended at `mem.drop(a)` on line 12 and is used on line 13
