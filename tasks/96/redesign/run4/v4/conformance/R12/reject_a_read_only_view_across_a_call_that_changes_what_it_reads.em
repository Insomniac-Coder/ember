#$ test: compile-fail
#$ rules: R12, R8
#$ help: call `bump()` after `r`'s last use
## R12 (rules.md lines 669-672): across a call through a function value, "no read-only view of what they
## change ... lives"; R8 (g) (rules.md lines 466-470) refuses the change at the call (E3021). `Counter`
## owns no memory, so the change is not a reshape. Message in the shape of rules.md section 3's line 984.

struct Counter:
    n: int = 0

fn main():
    c = Counter()
    bump = fn() changes c: c.n += 1
    r = ref c.n
    bump()    #$ error[E3021]: `bump()` (line 15) changes `c` (`fn() changes c`), which `r` reads (`r = ref c.n`, line 14), and `r` is used on line 16; call `bump()` after `r`'s last use
    println(r)
