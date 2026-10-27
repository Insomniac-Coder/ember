#$ test: compile-fail
#$ rules: R12, R8
#$ help: call `bump()` after `w`'s last use
## R12 (rules.md lines 553-556): across a call through a function value, no `ref mut` name of a place it
## changes lives; R8 (f) (rules.md lines 376-381) refuses the change through another name (E3022).
## Message in the shape of rules.md section 3's line 834.

struct Counter:
    n: int = 0

fn main():
    c = Counter()
    bump = fn() changes c: c.n += 1
    w = ref mut c.n
    bump()    #$ error[E3022]: `w` is the only name for `c.n` until its last use on line 16, and `bump()` (line 15) changes `c` (`fn() changes c`); call `bump()` after `w`'s last use
    w += 10
    println(c.n)
